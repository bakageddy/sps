use duckdb::Connection;

use crate::{
    handlers::types::{CPUMemoryDumpSummary, CPUMemoryPoint, ProcessSeries, ProcessUsage},
    store::{error::Error, tables::Tables},
};

pub fn get_cpu_memory_summary(cnx: &Connection) -> Result<Vec<CPUMemoryDumpSummary>, Error> {
    let query = format!(
        "(SELECT {0}.timestamp, MAX({0}.total) AS TotalCPU, MAX({1}.total) AS TotalMemory FROM {0} INNER JOIN {1} ON {0}.timestamp = {1}.timestamp GROUP BY {0}.timestamp ORDER BY {0}.timestamp) UNION ALL (SELECT timestamp, MAX({2}.total_cpu) AS TotalCPU, MAX({2}.total_mem) AS TotalMemory FROM {2} GROUP BY timestamp ORDER BY timestamp)",
        Tables::WindowsCPUStats.into_str(),
        Tables::WindowsMemoryStats.into_str(),
        Tables::LinuxStats.into_str(),
    );

    let mut stmt = cnx.prepare_cached(&query)?;
    let mut rows = stmt.query([])?;
    let mut dump_summaries = Vec::new();
    while let Some(row) = rows.next()? {
        let dump_summary = CPUMemoryDumpSummary {
            timestamp: row.get(0)?,
            total_cpu: row.get(1)?,
            total_memory: row.get(2)?,
        };
        dump_summaries.push(dump_summary);
    }

    Ok(dump_summaries)
}

pub fn get_cpu_processes(cnx: &Connection, timestamp: u64) -> Result<Vec<ProcessUsage>, Error> {
    let query = format!(
        "(SELECT {0}.pid as pid, {0}.name as name, coalesce(null) as user, {0}.cpu as cpu, {0}.path as path FROM {0} WHERE {0}.timestamp = $1) UNION ALL (SELECT {1}.pid as pid, {1}.name as name, {1}.user as user, {1}.cpu as cpu, {1}.path FROM {1} WHERE {1}.timestamp = $1)",
        Tables::WindowsCPUStats.into_str(),
        Tables::LinuxStats.into_str()
    );

    let mut stmt = cnx.prepare_cached(&query)?;
    let mut rows = stmt.query([timestamp])?;
    let mut procs = Vec::new();
    while let Some(row) = rows.next()? {
        let proc = ProcessUsage {
            pid: row.get(0)?,
            name: row.get(1)?,
            user: row.get(2)?,
            value: row.get(3)?,
            path: row.get(4)?,
        };
        procs.push(proc);
    }
    Ok(procs)
}

pub fn get_mem_processes(cnx: &Connection, timestamp: u64) -> Result<Vec<ProcessUsage>, Error> {
    let query = format!(
        "(SELECT {0}.pid as pid, {0}.name as name, coalesce(null) as user, {0}.mem as mem, {0}.path as path FROM {0} WHERE {0}.timestamp = $1) UNION ALL (SELECT {1}.pid as pid, {1}.name as name, {1}.user as user, {1}.mem as mem, {1}.path FROM {1} WHERE {1}.timestamp = $1)",
        Tables::WindowsMemoryStats.into_str(),
        Tables::LinuxStats.into_str()
    );

    let mut stmt = cnx.prepare_cached(&query)?;
    let mut rows = stmt.query([timestamp])?;
    let mut procs = Vec::new();
    while let Some(row) = rows.next()? {
        let proc = ProcessUsage {
            pid: row.get(0)?,
            name: row.get(1)?,
            user: row.get(2)?,
            value: row.get(3)?,
            path: row.get(4)?,
        };
        procs.push(proc);
    }
    Ok(procs)
}

pub fn get_cpumem_series(cnx: &Connection, pid: u64) -> Result<ProcessSeries, Error> {
    let cpu_series_query = format!(
        "(SELECT {0}.timestamp, {0}.cpu FROM {0} WHERE {0}.pid = $1 ORDER BY {0}.timestamp) UNION ALL (SELECT {1}.timestamp, {1}.cpu FROM {1} WHERE {1}.pid = $1 ORDER BY {1}.timestamp)",
        Tables::WindowsCPUStats.into_str(),
        Tables::LinuxStats.into_str(),
    );
    let mem_series_query = format!(
        "(SELECT {0}.timestamp, {0}.mem FROM {0} WHERE {0}.pid = $1 ORDER BY {0}.timestamp) UNION ALL (SELECT {1}.timestamp, {1}.mem FROM {1} WHERE {1}.pid = $1 ORDER BY {1}.timestamp)",
        Tables::WindowsMemoryStats.into_str(),
        Tables::LinuxStats.into_str(),
    );

    let mut cpu_stmt = cnx.prepare_cached(&cpu_series_query)?;
    let mut mem_stmt = cnx.prepare_cached(&mem_series_query)?;

    let mut cpu_rows = cpu_stmt.query([pid])?;
    let mut mem_rows = mem_stmt.query([pid])?;

    let mut cpu = Vec::new();
    let mut mem = Vec::new();

    while let Some(point) = cpu_rows.next()? {
        let point = CPUMemoryPoint {
            timestamp: point.get(0)?,
            value: point.get(1)?,
        };
        cpu.push(point);
    }

    while let Some(point) = mem_rows.next()? {
        let point = CPUMemoryPoint {
            timestamp: point.get(0)?,
            value: point.get(1)?,
        };
        mem.push(point);
    }

    Ok(ProcessSeries { cpu, memory: mem })
}

pub fn get_cpumem_path_series(
    cnx: &Connection,
    path: Option<String>,
    name: Option<String>,
) -> Result<ProcessSeries, Error> {
    let cpu_query = format!(
        "(SELECT {0}.timestamp, SUM({0}.cpu) FROM {0} WHERE {0}.path = $1 OR {0}.name = $2 GROUP BY {0}.path, {0}.name, {0}.timestamp ORDER BY {0}.timestamp ) UNION ALL (SELECT {1}.timestamp, SUM({1}.cpu) FROM {1} WHERE {1}.path = $1 OR {1}.name = $2 GROUP BY {1}.path, {1}.name, {1}.timestamp ORDER BY {1}.timestamp)",
        Tables::WindowsCPUStats.into_str(),
        Tables::LinuxStats.into_str()
    );

    let mem_query = format!(
        "(SELECT {0}.timestamp, SUM({0}.mem) FROM {0} WHERE {0}.path = $1 OR {0}.name = $2 GROUP BY {0}.path, {0}.name, {0}.timestamp ORDER BY {0}.timestamp ) UNION ALL (SELECT {1}.timestamp, SUM({1}.mem) FROM {1} WHERE {1}.path = $1 OR {1}.name = $2 GROUP BY {1}.path, {1}.name, {1}.timestamp ORDER BY {1}.timestamp)",
        Tables::WindowsMemoryStats.into_str(),
        Tables::LinuxStats.into_str()
    );

    let mut cpu_stmt = cnx.prepare_cached(&cpu_query)?;
    let mut mem_stmt = cnx.prepare_cached(&mem_query)?;

    let mut cpu_rows = cpu_stmt.query([path.as_ref(), name.as_ref()])?;
    let mut mem_rows = mem_stmt.query([path.as_ref(), name.as_ref()])?;

    let mut cpu = Vec::new();
    let mut mem = Vec::new();
    while let Some(row) = cpu_rows.next()? {
        let point = CPUMemoryPoint {
            timestamp: row.get(0)?,
            value: row.get(1)?,
        };

        cpu.push(point);
    }

    while let Some(row) = mem_rows.next()? {
        let point = CPUMemoryPoint {
            timestamp: row.get(0)?,
            value: row.get(1)?,
        };
        mem.push(point);
    }

    Ok(ProcessSeries { cpu, memory: mem })
}

pub fn get_cpumem_cpu_total_series(cnx: &Connection) -> Result<Vec<CPUMemoryPoint>, Error> {
    let query = format!(
        "(SELECT {0}.timestamp, {0}.total AS TotalCPU FROM {0} GROUP BY {0}.timestamp, {0}.total ORDER BY {0}.timestamp) UNION ALL (SELECT {1}.timestamp, {1}.total_cpu AS TotalCPU FROM {1} GROUP BY {1}.timestamp, {1}.total_cpu ORDER BY {1}.timestamp)",
        Tables::WindowsCPUStats.into_str(),
        Tables::LinuxStats.into_str()
    );

    let mut stmt = cnx.prepare_cached(&query)?;
    let mut rows = stmt.query([])?;
    let mut points = Vec::new();
    while let Some(row) = rows.next()? {
        let point = CPUMemoryPoint {
            timestamp: row.get(0)?,
            value: row.get(1)?,
        };
        points.push(point);
    }
    Ok(points)
}

pub fn get_cpumem_mem_total_series(cnx: &Connection) -> Result<Vec<CPUMemoryPoint>, Error> {
    let query = format!(
        "(SELECT {0}.timestamp, {0}.total AS TotalCPU FROM {0} GROUP BY {0}.timestamp, {0}.total ORDER BY {0}.timestamp) UNION ALL (SELECT {1}.timestamp, {1}.total_mem AS TotalCPU FROM {1} GROUP BY {1}.timestamp, {1}.total_mem ORDER BY {1}.timestamp)",
        Tables::WindowsMemoryStats.into_str(),
        Tables::LinuxStats.into_str()
    );

    let mut stmt = cnx.prepare_cached(&query)?;
    let mut rows = stmt.query([])?;
    let mut points = Vec::new();
    while let Some(row) = rows.next()? {
        let point = CPUMemoryPoint {
            timestamp: row.get(0)?,
            value: row.get(1)?,
        };
        points.push(point);
    }
    Ok(points)
}
