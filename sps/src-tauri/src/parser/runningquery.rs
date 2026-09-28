use error::Error;
use std::str::Utf8Error;

use crate::{
    parser::{
        TableKind,
        query::{BlockingQuery, MSSQLQuery, PGSQLQuery, SPWho2Query},
        tokenizer::{Parser, Tokenizer},
    },
    util,
};

use time::format_description::BorrowedFormatItem;
use time::macros::format_description;

const RUNNINGQUERY_TIME_FORMAT: &[BorrowedFormatItem] =
    format_description!("[hour]:[minute]:[second].[subsecond]");
const RUNNINGQUERY_DATE_FORMAT: &[BorrowedFormatItem] = format_description!("[day]-[month]-[year]");

#[derive(Debug)]
pub struct RunningQueryParser<'a>(&'a str, ParserState);

#[derive(Debug)]
enum ParserState {
    Initial,
    Header,
    QueryTable,
}

impl<'a> Iterator for RunningQueryParser<'a> {
    type Item = Result<RunningQueryTable<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut tok = Tokenizer::new(self.0);
        tok.skip_whitespace();
        if tok.is_empty() {
            return None;
        }

        self.1 = ParserState::Initial;
        while let Some(line) = tok.peek_line() {
            if line.trim_start().starts_with("[") && line.trim_end().ends_with("::") {
                self.1 = ParserState::Header;
                break;
            }
            let _ = tok.get_line()?;
        }

        if let ParserState::Initial = self.1 {
            return None;
        }

        let header = tok.get_line()?;
        let timestamp = Self::parse_header(header);
        if let Err(e) = timestamp {
            self.0 = tok.remaining();
            return Some(Err(e));
        }
        let timestamp = timestamp.unwrap();

        tok.skip_whitespace();
        if tok.is_empty() {
            return None;
        }

        if !tok.peek("|") {
            self.0 = tok.remaining();
            return Some(Err(Error::TableNotFound));
        }

        self.1 = ParserState::QueryTable;
        let mut queries = Vec::new();
        loop {
            let mut table_header = Vec::new();
            let mut table_header_count = 0;
            while let Some(line) = tok.peek_line() {
                if table_header_count == 5 {
                    break;
                }

                if line.trim_start().starts_with("|") {
                    table_header.push(line);
                    table_header_count += 1;
                    let _ = tok.get_line()?;
                } else {
                    break;
                }
            }

            if table_header.len() != 5 {
                self.0 = tok.remaining();
                return Some(Err(Error::MalformedTableHeader));
            }

            let table_kind = if let Some(kind) = TableKind::detect_table(&table_header) {
                kind
            } else {
                self.0 = tok.remaining();
                return Some(Err(Error::UnableToDetectTableKind));
            };

            while let Some(line) = tok.peek_line() {
                if !line.trim_start().starts_with("|") {
                    break;
                }

                match table_kind {
                    TableKind::PGSQLRunningQuery => match PGSQLQuery::parse(line) {
                        Ok(query) => queries.push(RunningQuery::PGSQL(query)),
                        Err(e) => {
                            self.0 = tok.remaining();
                            return Some(Err(Error::from(e)));
                        }
                    },
                    TableKind::MSSQLRunningQuery => match MSSQLQuery::parse(line) {
                        Ok(query) => queries.push(RunningQuery::MSSQL(query)),
                        Err(e) => {
                            self.0 = tok.remaining();
                            return Some(Err(Error::from(e)));
                        }
                    },
                    TableKind::MSSQLBlockingQuery => match BlockingQuery::parse(line) {
                        Ok(query) => queries.push(RunningQuery::Blocking(query)),
                        Err(e) => {
                            self.0 = tok.remaining();
                            return Some(Err(Error::from(e)));
                        }
                    },

                    TableKind::MSSQLSPWho2 => match SPWho2Query::parse(line) {
                        Ok(query) => queries.push(RunningQuery::SPWho2(query)),
                        Err(e) => {
                            self.0 = tok.remaining();
                            return Some(Err(Error::from(e)));
                        }
                    },
                }

                let _ = tok.get_line()?;
            }

            if let TableKind::MSSQLRunningQuery = table_kind {
                tok.skip_whitespace();
                continue;
            } else {
                break;
            }
        }

        self.0 = tok.remaining();
        Some(Ok(RunningQueryTable { timestamp, queries }))
    }
}

impl<'a> RunningQueryParser<'a> {
    pub fn new(data: &'a str) -> Self {
        Self(data, ParserState::Initial)
    }

    pub fn parse_header(header: &'a str) -> Result<u64, Error> {
        let mut tok = Tokenizer::new(header);
        let time = tok.take_within("[", "]")?;
        let date = tok.take_within("[", "]")?;

        let timestamp = util::unix_timestamp_millis(
            time,
            date,
            RUNNINGQUERY_TIME_FORMAT,
            RUNNINGQUERY_DATE_FORMAT,
        )?;

        Ok(timestamp)
    }
}

impl<'a> TryFrom<&'a [u8]> for RunningQueryParser<'a> {
    type Error = Utf8Error;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        let data = std::str::from_utf8(value)?;
        Ok(Self::new(data))
    }
}

#[derive(Debug)]
pub struct RunningQueryTable<'a> {
    pub timestamp: u64,
    pub queries: Vec<RunningQuery<'a>>,
}

#[derive(Debug)]
pub enum RunningQuery<'a> {
    PGSQL(PGSQLQuery<'a>),
    MSSQL(MSSQLQuery<'a>),
    Blocking(BlockingQuery<'a>),
    SPWho2(SPWho2Query<'a>),
}

pub mod error {
    use crate::parser::query;
    use crate::parser::tokenizer;
    use crate::types::TimestampError;

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Invalid Format in parsing: {0}")]
        InvalidFormat(#[from] tokenizer::error::Error),

        #[error("Invalid Timestamp in parsing: {0}")]
        Timestamp(#[from] TimestampError),

        #[error("Table not found")]
        TableNotFound,

        #[error("Table Header malformed")]
        MalformedTableHeader,

        #[error("Unable to detect table kind")]
        UnableToDetectTableKind,

        #[error("Query parse: {0}")]
        QueryParse(#[from] query::error::Error),
    }
}

#[cfg(test)]
pub mod test {
    use std::ops::Deref;

use crate::{parser::runningquery::{RunningQueryParser, error::Error}, util};

    #[test]
    fn test_runningqueries() {
        let map = util::map_file("test/runningqueries/runningqueries0.txt").unwrap();
        let parser = RunningQueryParser::try_from(map.deref()).unwrap();

        let mut count = 0;
        for result in parser {
            if let Err(Error::TableNotFound) = result {
                continue;
            }

            assert!(result.is_ok(), "Error during parsing: {}", result.unwrap_err());
            let table = result.unwrap();
            assert_ne!(table.timestamp, 0);
            // assert_ne!(table.queries.len(), 0);
            count += 1;
        }
        assert_eq!(count, 144);

        let map = util::map_file("test/runningqueries/runningqueries1.txt").unwrap();
        let parser = RunningQueryParser::try_from(map.deref()).unwrap();

        let mut count = 0;
        for result in parser {
            if let Err(Error::TableNotFound) = result {
                continue;
            }

            assert!(result.is_ok(), "Error during parsing: {}", result.unwrap_err());
            let table = result.unwrap();
            assert_ne!(table.timestamp, 0);
            // assert_ne!(table.queries.len(), 0);
            count += 1;
        }
        assert_eq!(count, 48);

        let map = util::map_file("test/runningqueries/runningqueries2.txt").unwrap();
        let parser = RunningQueryParser::try_from(map.deref()).unwrap();

        let mut count = 0;
        for result in parser {
            if let Err(Error::TableNotFound) = result {
                continue;
            }

            assert!(result.is_ok(), "Error during parsing: {}", result.unwrap_err());
            let table = result.unwrap();
            assert_ne!(table.timestamp, 0);
            // assert_ne!(table.queries.len(), 0);
            count += 1;
        }
        assert_eq!(count, 60);

        let map = util::map_file("test/runningqueries/runningqueries3.txt").unwrap();
        let parser = RunningQueryParser::try_from(map.deref()).unwrap();

        let mut count = 0;
        for result in parser {
            if let Err(Error::TableNotFound) = result {
                continue;
            }

            assert!(result.is_ok(), "Error during parsing: {}", result.unwrap_err());
            let table = result.unwrap();
            assert_ne!(table.timestamp, 0);
            // assert_ne!(table.queries.len(), 0);
            count += 1;
        }
        assert_eq!(count, 61);

        let map = util::map_file("test/runningqueries/runningqueries4.txt").unwrap();
        let parser = RunningQueryParser::try_from(map.deref()).unwrap();

        let mut count = 0;
        for result in parser {
            if let Err(Error::TableNotFound) = result {
                continue;
            }

            assert!(result.is_ok(), "Error during parsing: {}", result.unwrap_err());
            let table = result.unwrap();
            assert_ne!(table.timestamp, 0);
            // assert_ne!(table.queries.len(), 0);
            count += 1;
        }
        assert_eq!(count, 74 * 2);

        let map = util::map_file("test/runningqueries/runningqueries5.txt").unwrap();
        let parser = RunningQueryParser::try_from(map.deref()).unwrap();

        let mut count = 0;
        for result in parser {
            if let Err(Error::TableNotFound) = result {
                continue;
            }

            assert!(result.is_ok(), "Error during parsing: {}", result.unwrap_err());
            let table = result.unwrap();
            assert_ne!(table.timestamp, 0);
            // assert_ne!(table.queries.len(), 0);
            count += 1;
        }
        assert_eq!(count, 105 + 106);
    }
}
