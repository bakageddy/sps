use crate::{
    handlers::types::{FragFlag, FragPoint, FragSnapshot},
    parser::query::FragmentationRatioEntry,
    store::{error::Error, tables::Tables},
};
use duckdb::Connection;

pub fn get_fragmentation_snapshots(cnx: &Connection) -> Result<Vec<FragSnapshot>, Error> {
    let query = format!(
        r"
        SELECT
            timestamp,
            COUNT(*),
            MAX(ratio),
            SUM(page_count)
        FROM {}
        GROUP BY timestamp
        ORDER BY timestamp
    ",
        Tables::FragmentationRatio
    );
    let mut stmt = cnx.prepare_cached(&query)?;
    stmt.query_and_then([], |row| {
        Ok(FragSnapshot {
            timestamp: row.get(0)?,
            indices: row.get(1)?,
            max_ratio: row.get(2)?,
            total_pages: row.get(3)?,
        })
    })?
    .collect()
}

pub fn get_fragmentation_indices(
    cnx: &Connection,
    timestamp: u64,
) -> Result<Vec<FragmentationRatioEntry<'static>>, Error> {
    let query = format!(
        r"
        SELECT
            schema,
            table_name,
            index_name,
            ratio,
            page_count
        FROM {}
        WHERE timestamp = ?
        ORDER BY ratio DESC
    ",
        Tables::FragmentationRatio
    );
    let mut stmt = cnx.prepare_cached(&query)?;
    stmt.query_and_then([timestamp], |row| {
        Ok(FragmentationRatioEntry {
            schema: row.get::<_, String>(0)?.into(),
            table: row.get::<_, String>(1)?.into(),
            index: row.get::<_, String>(2)?.into(),
            ratio: row.get(3)?,
            page_count: row.get(4)?,
        })
    })?
    .collect()
}

pub fn get_fragmentation_series(
    cnx: &Connection,
    schema: String,
    table: String,
    index: String,
) -> Result<Vec<FragPoint>, Error> {
    let query = format!(
        r"
        SELECT
            timestamp,
            ratio,
            page_count
        FROM {}
        WHERE schema = ? AND table_name = ? AND index_name = ?
        ORDER BY timestamp
    ",
        Tables::FragmentationRatio
    );
    let mut stmt = cnx.prepare_cached(&query)?;
    stmt.query_and_then([schema, table, index], |row| {
        Ok(FragPoint {
            timestamp: row.get(0)?,
            ratio: row.get(1)?,
            page_count: row.get(2)?,
        })
    })?
    .collect()
}

pub fn get_fragmentation_flagged(
    cnx: &Connection,
    min_pages: u64,
    min_ratio: f32,
) -> Result<Vec<FragFlag>, Error> {
    let query = format!(r"
        SELECT
            schema,
            table_name,
            index_name,
            COUNT(*) as captures,
            COUNT(*) FILTER (WHERE page_count >= $1 AND ratio >= $2) as flagged_captures,
            MIN(timestamp) FILTER (WHERE page_count >= $1 AND ratio >= $2),
            MAX(timestamp) FILTER (WHERE page_count >= $1 AND ratio >= $2),
            LAST(ratio),
            LAST(page_count),
            MAX(ratio),
        FROM {}
        GROUP BY schema, table_name, index_name
        ORDER BY timestamp
    ", Tables::FragmentationRatio);
    let mut stmt = cnx.prepare_cached(&query)?;
    stmt.query_and_then((min_pages, min_ratio), |row| {
        Ok(FragFlag {
            schema: row.get(0)?,
            table: row.get(1)?,
            index: row.get(2)?,
            captures: row.get(3)?,
            flagged_captures: row.get(4)?,
            first_flagged: row.get(5)?,
            last_flagged: row.get(6)?,
            latest_ratio: row.get(7)?,
            latest_pages: row.get(8)?,
            max_ratio: row.get(9)?
        })
    })?.collect()
}
