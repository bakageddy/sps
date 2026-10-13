use crate::store::error::Error;
use crate::store::tables::Tables;
use duckdb::{Connection, OptionalExt};

pub fn get_timezone_info(cnx: &Connection) -> Result<Option<String>, Error> {
    let query = format!(
        "SELECT val FROM {} WHERE key = $1 LIMIT 1",
        Tables::HealthMeter
    );
    let mut stmt = cnx.prepare_cached(&query)?;
    Ok(stmt
        .query_one(["Server Time"], |row| row.get::<_, Option<String>>(0))
        .optional()?
        .flatten())
}

pub fn get_all_server_time(cnx: &Connection) -> Result<Vec<Option<String>>, Error> {
    let query = format!("SELECT val FROM {} WHERE key = $1", Tables::HealthMeter);

    let mut stmt = cnx.prepare_cached(&query)?;
    let mut values = stmt.query(["Server Time"])?;
    let mut result = Vec::new();
    while let Some(row) = values.next()? {
        result.push(row.get(0)?);
    }
    Ok(result)
}
