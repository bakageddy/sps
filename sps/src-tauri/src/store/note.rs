use duckdb::Connection;

use crate::{handlers::types::Note, store::error::Error};

pub fn get_notes(cnx: &Connection) -> Result<Vec<Note>, Error> {
    let query = "SELECT * FROM notes";
    let mut stmt = cnx.prepare_cached(query)?;
    stmt.query_and_then([], |row| {
        Ok(Note {
            created_at: row.get(0)?,
            updated_at: row.get(1)?,
            text: row.get(2)?,
            route: row.get(3)?,
        })
    })?
    .into_iter()
    .collect()
}

pub fn upsert(cnx: &Connection, note: Note) -> Result<(), Error> {
    let query =
        "INSERT OR REPLACE INTO notes (created_at, updated_at, text, route) VALUES($1, $2, $3, $4)";
    let mut stmt = cnx.prepare_cached(query)?;
    stmt.insert((note.created_at, note.updated_at, note.text, note.route))?;
    Ok(())
}

pub fn delete(cnx: &Connection, created_at: u64) -> Result<(), Error> {
    let query = "DELETE FROM notes WHERE created_at = $1";
    let mut stmt = cnx.prepare_cached(query)?;
    stmt.execute([created_at])?;
    Ok(())
}
