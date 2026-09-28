use self::error::Error;
use crate::parser::TableKind;
use crate::parser::query::BlockingQuery;
use crate::parser::query::MSSQLQuery;
use crate::parser::query::PGSQLQuery;
use crate::parser::tokenizer::Parser;
use crate::parser::tokenizer::Tokenizer;
use crate::util;
use time::format_description::BorrowedFormatItem;
use time::macros::format_description;

const STUCKQUERY_TIME_FORMAT: &[BorrowedFormatItem] =
    format_description!("[hour]:[minute]:[second].[subsecond]");
const STUCKQUERY_DATE_FORMAT: &[BorrowedFormatItem] = format_description!("[day]-[month]-[year]");

#[derive(Debug)]
pub struct StuckQueryParser<'a>(&'a str, ParserState);

#[derive(Debug)]
enum ParserState {
    Initial,
    Header,
    QueryTable,
}

impl<'a> StuckQueryParser<'a> {
    pub fn new(data: &'a str) -> Self {
        Self(data, ParserState::Initial)
    }

    pub fn parse_header(data: &'a str) -> Result<u64, Error> {
        let mut tok = Tokenizer::new(data);
        let time = tok.take_within("[", "]")?;
        let date = tok.take_within("[", "]")?;

        let timestamp = util::unix_timestamp_millis(
            time,
            date,
            STUCKQUERY_TIME_FORMAT,
            STUCKQUERY_DATE_FORMAT,
        )?;

        Ok(timestamp)
    }
}

impl<'a> Iterator for StuckQueryParser<'a> {
    type Item = Result<StuckQueryTable<'a>, Error>;

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
                        Ok(query) => queries.push(StuckQuery::PGSQL(query)),
                        Err(e) => {
                            self.0 = tok.remaining();
                            return Some(Err(Error::from(e)));
                        }
                    },
                    TableKind::MSSQLRunningQuery => match MSSQLQuery::parse(line) {
                        Ok(query) => queries.push(StuckQuery::MSSQL(query)),
                        Err(e) => {
                            self.0 = tok.remaining();
                            return Some(Err(Error::from(e)));
                        }
                    },
                    TableKind::MSSQLBlockingQuery => match BlockingQuery::parse(line) {
                        Ok(query) => queries.push(StuckQuery::Blocking(query)),
                        Err(e) => {
                            self.0 = tok.remaining();
                            return Some(Err(Error::from(e)));
                        }
                    },

                    TableKind::MSSQLSPWho2 => {
                        self.0 = tok.remaining();
                        return Some(Err(Error::InvalidQueryType));
                    }
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
        Some(Ok(StuckQueryTable { timestamp, queries }))
    }
}

#[derive(Debug)]
pub struct StuckQueryTable<'a> {
    pub timestamp: u64,
    pub queries: Vec<StuckQuery<'a>>,
}

#[derive(Debug)]
pub enum StuckQuery<'a> {
    PGSQL(PGSQLQuery<'a>),
    MSSQL(MSSQLQuery<'a>),
    Blocking(BlockingQuery<'a>),
}

pub mod error {
    use crate::parser::{query, tokenizer};
    use crate::types::TimestampError;

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Invalid Format in parsing: {0}")]
        InvalidFormat(#[from] tokenizer::error::Error),

        #[error("Invalid Timestamp in parsing: {0}")]
        Timestamp(#[from] TimestampError),

        #[error("Query parse error: {0}")]
        QueryParse(#[from] query::error::Error),

        #[error("Table not found")]
        TableNotFound,

        #[error("Table Header malformed")]
        MalformedTableHeader,

        #[error("Unable to detect table kind")]
        UnableToDetectTableKind,

        #[error("Invalid Query Type in stuckquery")]
        InvalidQueryType,
    }
}

impl<'a> TryFrom<&'a [u8]> for StuckQueryParser<'a> {
    type Error = std::str::Utf8Error;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        let data = std::str::from_utf8(value)?;
        Ok(Self::new(data))
    }
}

#[cfg(test)]
pub mod test {
    use super::StuckQueryParser;
    use crate::util;
    use std::ops::Deref;

    #[test]
    fn stuckquery_pgsql_single_table() {
        let map = util::map_file("test/stuckqueries/stuckquery_pgsql_single_table.txt").unwrap();
        let mut parser = StuckQueryParser::try_from(map.deref()).unwrap();
        let result = parser.next();
        assert!(result.is_some());
        let result = result.unwrap();
        assert!(
            result.is_ok(),
            "Error during parsing: {}",
            result.unwrap_err()
        );
        let result = result.unwrap();
        assert_ne!(result.timestamp, 0);
        assert_eq!(result.queries.len(), 35);
    }

    #[test]
    fn stuckquery_pgsql_full_file() {
        let map = util::map_file("test/stuckqueries/stuckquery_pgsql.txt").unwrap();
        let parser = StuckQueryParser::try_from(map.deref()).unwrap();
        let mut count = 0;
        for result in parser {
            assert!(
                result.is_ok(),
                "Error during parsing: {}",
                result.unwrap_err()
            );
            count += 1;
        }
        assert_eq!(count, 41);

        let map = util::map_file("test/stuckqueries/stuckquery_pgsql_2.txt").unwrap();
        let parser = StuckQueryParser::try_from(map.deref()).unwrap();
        let mut count = 0;
        for result in parser {
            assert!(
                result.is_ok(),
                "Error during parsing: {}",
                result.unwrap_err()
            );
            count += 1;
        }
        assert_eq!(count, 8);
    }

    #[test]
    fn stuckquery_mssql_single_running_query_table() {
        let map = util::map_file("test/stuckqueries/stuckquery_mssql_single_table.txt").unwrap();
        let parser = StuckQueryParser::try_from(map.deref()).unwrap();
        for result in parser {
            assert!(
                result.is_ok(),
                "Error during parsing: {}",
                result.unwrap_err()
            );
            let result = result.unwrap();
            assert_eq!(result.queries.len(), 18);
            assert_ne!(result.timestamp, 0);
        }
    }

    #[test]
    fn stuckquery_mssql_full_file() {
        let map = util::map_file("test/stuckqueries/stuckquery_mssql.txt").unwrap();
        let parser = StuckQueryParser::try_from(map.deref()).unwrap();
        let mut count = 0;
        for result in parser {
            assert!(
                result.is_ok(),
                "Error during parsing: {}",
                result.unwrap_err()
            );
            count += 1;
        }
        assert_eq!(count, 61);

        let map = util::map_file("test/stuckqueries/stuckquery_mssql_2.txt").unwrap();
        let parser = StuckQueryParser::try_from(map.deref()).unwrap();
        let mut count = 0;
        for result in parser {
            assert!(
                result.is_ok(),
                "Error during parsing: {}",
                result.unwrap_err()
            );
            count += 1;
        }
        assert_eq!(count, 55);
    }
}
