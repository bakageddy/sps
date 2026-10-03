use crate::{
    handlers::types::{SQLColumn, SQLResult, SQLTable},
    store::error::Error,
};
use duckdb::Connection;
use std::path::Path;

pub fn get_schema(cnx: &Connection) -> Result<Vec<SQLTable>, Error> {
    let query = r"
        SELECT table_name
        FROM duckdb_tables()
    ";
    let mut stmt = cnx.prepare_cached(query)?;
    let mut rows = stmt.query([])?;
    let mut tables = Vec::new();
    while let Some(row) = rows.next()? {
        let table: String = row.get(0)?;
        let count_query = format!("SELECT COUNT(*) FROM {}", table);
        let columns = get_table_columns(cnx, &table)?;
        let count = cnx.query_row(&count_query, [], |row| row.get::<_, u64>(0))?;
        tables.push(SQLTable {
            name: table,
            rows: count,
            columns,
        });
    }
    Ok(tables)
}

pub fn get_table_columns(
    cnx: &Connection,
    table: impl AsRef<str>,
) -> Result<Vec<SQLColumn>, Error> {
    let query = r"
        SELECT
            column_name,
            data_type,
            is_nullable,
            column_default
        FROM duckdb_columns() as col
        INNER JOIN duckdb_types() as t
            ON col.database_name = t.database_name AND
            col.schema_name = t.schema_name AND
            col.data_type_id = t.type_oid
        WHERE table_name = $1
        ORDER BY column_index
    ";
    let mut stmt = cnx.prepare_cached(query)?;
    stmt.query_and_then([table.as_ref()], |row| {
        Ok(SQLColumn {
            name: row.get(0)?,
            r#type: row.get(1)?,
            nullable: row.get(2)?,
            default_value: row.get(3)?,
        })
    })?
    .into_iter()
    .collect()
}

pub fn execute_query(
    cnx: &Connection,
    query: impl AsRef<str>,
    limit: u64,
    offset: u64,
) -> Result<Option<SQLResult>, Error> {
    let exec = format!(
        "SELECT COLUMNS(*)::VARCHAR FROM ({}) LIMIT $1 OFFSET $2",
        query.as_ref()
    );
    let mut stmt = cnx.prepare_cached(&exec)?;
    let start = std::time::Instant::now();
    let mut rows = stmt.query([limit, offset])?;
    let column_count = rows.as_ref().unwrap().column_count();
    let mut result = Vec::new();
    while let Some(row) = rows.next()? {
        let mut temp = Vec::new();
        for i in 0..column_count {
            let val: Option<String> = row.get(i)?;
            temp.push(val);
        }
        result.push(temp);
    }
    let end = std::time::Instant::now();
    let delta = end - start;

    if result.is_empty() {
        return Ok(None);
    }

    let mut columns = Vec::new();
    for i in 0..column_count {
        columns.push(stmt.column_name(i)?.to_string());
    }

    Ok(Some(SQLResult {
        columns,
        rows: result,
        elapsed_ms: delta.as_millis(),
    }))
}

pub fn export(
    cnx: &Connection,
    query: impl AsRef<str>,
    path: impl AsRef<Path>,
) -> Result<usize, Error> {
    let exec = format!(
        "COPY ({}) TO '{}' (FORMAT CSV, HEADER)",
        query.as_ref(),
        path.as_ref().display()
    );
    let mut stmt = cnx.prepare_cached(&exec)?;
    stmt.execute([]).map_err(Error::from)
}
