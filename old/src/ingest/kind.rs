use crate::error::kind::Error;
use memchr::memmem::find;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum DBKind {
    PGSQL,
    MSSQL,
}

#[derive(Debug, Clone)]
pub enum OSKind {
    Linux,
    Windows,
    MacOS,
}

impl DBKind {
    fn contains(haystack: &[u8], needle: &[u8]) -> bool {
        find(haystack, needle).is_some()
    }

    pub fn detect(body: &[u8]) -> Result<Self, Error> {
        if Self::contains(body, b"Query Time (s)") {
            Ok(Self::PGSQL)
        } else if Self::contains(body, b"Logical Reads")
            || Self::contains(body, b"Wait Resource")
            || Self::contains(body, b"CPUTime")
        {
            Ok(Self::MSSQL)
        } else {
            Err(Error::Detection)
        }
    }
}
