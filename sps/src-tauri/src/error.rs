use crate::store;
use crate::parser;

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO Error: {0}")]
    IO(#[from] std::io::Error),
    #[error("Parsing Error")]
    Parse(#[from] parser::error::Error),
    #[error("Database Error: {0}")]
    Store(#[from] store::error::Error),
}
