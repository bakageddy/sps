use std::str::Utf8Error;

use time::format_description::BorrowedFormatItem;
use time::macros::format_description;
use time_tz::Tz;

use self::error::Error;
use crate::{
    parser::{
        Context, TableKind,
        query::FragmentationRatioEntry,
        tokenizer::{Parser, Tokenizer},
    },
    util,
};

const FRAGMENTATION_RATIO_TIME_FORMAT: &[BorrowedFormatItem] =
    format_description!("[hour]:[minute]:[second].[subsecond]");
const FRAGMENTATION_RATIO_DATE_FORMAT: &[BorrowedFormatItem] =
    format_description!("[day]-[month]-[year]");

#[derive(Debug)]
pub struct FragmentationRatioTable<'a> {
    pub values: Vec<FragmentationRatioEntry<'a>>,
    pub timestamp: u64,
}

#[derive(Debug)]
pub struct FragmentationRatioParser<'a>(&'a str, ParserState, Context);

#[derive(Debug)]
pub enum ParserState {
    Initial,
    Timestamp,
    TableKind,
    Entry,
}

impl<'a> FragmentationRatioParser<'a> {
    pub fn new(data: &'a str) -> Self {
        Self(data, ParserState::Initial, Context::EMPTY)
    }

    pub fn with_context(data: &'a str, ctx: Context) -> Self {
        Self(data, ParserState::Initial, ctx)
    }

    pub fn from_bytes_with_context(data: &'a [u8], ctx: Context) -> Result<Self, Utf8Error> {
        let data = std::str::from_utf8(data)?;
        Ok(Self::with_context(data, ctx))
    }

    pub fn parse_timestamp(tok: &mut Tokenizer, zone: Option<&'static Tz>) -> Result<u64, Error> {
        let time = tok.take_within("[", "]")?;
        let date = tok.take_within("[", "]")?;
        let timestamp = util::unix_timestamp_millis(
            time,
            date,
            FRAGMENTATION_RATIO_TIME_FORMAT,
            FRAGMENTATION_RATIO_DATE_FORMAT,
            zone,
        )?;
        Ok(timestamp)
    }
}

impl<'a> TryFrom<&'a [u8]> for FragmentationRatioParser<'a> {
    type Error = Utf8Error;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        let data = std::str::from_utf8(value)?;
        Ok(Self::new(data))
    }
}

impl<'a> Iterator for FragmentationRatioParser<'a> {
    type Item = Result<FragmentationRatioTable<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut tok = Tokenizer::new(self.0);
        tok.skip_whitespace();
        if tok.is_empty() {
            return None;
        }

        self.1 = ParserState::Initial;
        while let Some(line) = tok.peek_line() {
            if line.trim_start().starts_with("[") && line.trim_end().ends_with("::") {
                self.1 = ParserState::Timestamp;
                break;
            }
        }

        if let ParserState::Initial = self.1 {
            self.0 = tok.remaining();
            return None;
        }

        let timestamp = match Self::parse_timestamp(&mut tok, self.2.zone) {
            Ok(x) => x,
            Err(e) => {
                self.0 = tok.remaining();
                return Some(Err(e));
            }
        };

        tok.get_line()?;
        tok.skip_whitespace();
        if !tok.peek("|") {
            self.0 = tok.remaining();
            return Some(Err(Error::MalformedTableHeader));
        }

        self.1 = ParserState::TableKind;
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

        let kind = if let Some(kind) = TableKind::detect_table(&table_header) {
            kind
        } else {
            self.0 = tok.remaining();
            return Some(Err(Error::UnableToDetectTableKind));
        };

        self.1 = ParserState::Entry;
        let mut entries = Vec::new();
        match kind {
            TableKind::MSSQLFragmentationRatio => {
                while let Some(line) = tok.peek_line() {
                    if !line.trim_start().starts_with("|") {
                        break;
                    }

                    match FragmentationRatioEntry::parse(line) {
                        Ok(entry) => {
                            entries.push(entry);
                            let _ = tok.get_line()?;
                        }
                        Err(e) => {
                            self.0 = tok.remaining();
                            return Some(Err(Error::from(e)));
                        }
                    }
                }
            }
            _ => {
                self.0 = tok.remaining();
                return Some(Err(Error::InvalidTableKind));
            }
        }

        self.0 = tok.remaining();
        Some(Ok(FragmentationRatioTable {
            values: entries,
            timestamp,
        }))
    }
}

pub mod error {
    use crate::{
        parser::{query, tokenizer},
        types,
    };

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Invalid Format: {0}")]
        InvalidFormat(#[from] tokenizer::error::Error),
        #[error("Invalid Timestamp: {0}")]
        InvalidTimestamp(#[from] types::TimestampError),
        #[error("Invalid Fragmentation Ratio Entry: {0}")]
        InvalidEntry(#[from] query::error::Error),
        #[error("Malformed Table Header")]
        MalformedTableHeader,
        #[error("Unable to detect Table Kind")]
        UnableToDetectTableKind,
        #[error("Invalid Table Kind found")]
        InvalidTableKind,
    }
}

#[cfg(test)]
pub mod test {
    use std::ops::Deref;

    use super::FragmentationRatioParser;
    use crate::util;

    #[test]
    fn fragmentation_ratio_full_file() {
        let map = util::map_file("test/FragmentationRatio/FragmentationRatio0.txt").unwrap();
        let parser = FragmentationRatioParser::try_from(map.deref()).unwrap();
        let mut count = 0;
        for result in parser {
            assert!(
                result.is_ok(),
                "Error during parsing: {}",
                result.unwrap_err()
            );
            let table = result.unwrap();
            assert_ne!(table.timestamp, 0);
            assert_eq!(table.values.len(), 7371);
            count += 1;
        }

        assert_eq!(count, 3);

        let map = util::map_file("test/FragmentationRatio/FragmentationRatio1.txt").unwrap();
        let parser = FragmentationRatioParser::try_from(map.deref()).unwrap();
        let mut count = 0;
        for result in parser {
            assert!(
                result.is_ok(),
                "Error during parsing: {}",
                result.unwrap_err()
            );
            let table = result.unwrap();
            assert_ne!(table.timestamp, 0);
            assert_eq!(table.values.len(), 7371);
            count += 1;
        }

        assert_eq!(count, 9);
    }
}
