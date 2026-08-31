#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Error during creating/acquiring connection pool: {0}")]
    R2D2(#[from] r2d2::Error),
    #[error("Error during quering database: {0}")]
    DuckDB(#[from] duckdb::Error),
}
