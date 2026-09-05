use std::borrow::Cow;

use error::Error;

use crate::parser::tokenizer::Tokenizer;

#[derive(Debug)]
pub struct StuckqueryParser<'a>(&'a str, ParserState);
impl<'a> StuckqueryParser<'a> {
    fn parse_header(header: &str) -> Result<u64, Error> {
        todo!()
    }
}

#[derive(Debug)]
enum ParserState {
    Initial,
    Header,
}

impl<'a> Iterator for StuckqueryParser<'a> {
    type Item = Result<Stuckquery<'a>, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        let mut tok = Tokenizer::new(self.0);
        if tok.is_empty() {
            return None;
        }

        self.1 = ParserState::Initial;
        while let Some(line) = tok.peek_line()
            && line.trim_start().starts_with("[")
        {
            if line.trim_end().ends_with("::") {
                self.1 = ParserState::Header;
                break;
            }
        }

        if !matches!(self.1, ParserState::Header) {}

        let header = tok.get_line()?;
        let timestamp = Self::parse_header(header);

        tok.remaining();

        todo!()
    }
}

#[derive(Debug)]
pub enum Stuckquery<'a> {
    PGSQL(PGSQLQuery<'a>),
    MSSQL(MSSQLQuery<'a>),
}

#[derive(Debug)]
pub struct PGSQLQuery<'a> {
    pub pid: u64,
    pub query_time: u64,
    pub txn_time: u64,
    pub db_name: Cow<'a, str>,
    pub state: PGSQLState,
    pub waiting: bool,
    pub query: Cow<'a, str>,
    pub state_change: u64,
    pub application_name: Cow<'a, str>,
    pub client_addr: Cow<'a, str>,
    pub client_host: Option<Cow<'a, str>>,
    pub client_port: Option<u16>,
}

#[derive(Debug)]
pub enum PGSQLState {
    Active,
    Idle,
}

#[derive(Debug)]
pub enum MSSQLQuery<'a> {
    // Blocking(BlockingQuery<'a>),
    Blocking,
    Running(RunningQuery<'a>),
}

#[derive(Debug)]
pub struct RunningQuery<'a> {
    pub session_id: u64,
    pub status: (),
    pub txn_id: u64,
    pub blocked_by: u64,
    pub wait_type: Cow<'a, str>,
    pub wait_resource: Cow<'a, str>,
    pub wait_time_ms: u64,
    pub cpu_time_ms: u64,
    pub logical_reads: u64,
    pub reads: u64,
    pub writes: u64,
    pub elapsed: u64,
    pub statement: Cow<'a, str>,
    pub command: Cow<'a, str>,
    pub login: Cow<'a, str>,
    pub host: Cow<'a, str>,
    pub db: Cow<'a, str>,
    pub program: Cow<'a, str>,
    pub host_process: Cow<'a, str>,
    pub last_request_end: u64,
    pub login_time: u64,
    pub open_txn: u64,
}

// pub enum MSSQLStatus {
// }
//
// #[derive(Debug)]
// pub struct BlockingQuery<'a> {}

pub mod error {
    use crate::{parser::tokenizer, types::TimestampError};

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Invalid Format: {0}")]
        InvalidFormat(#[from] tokenizer::error::Error),
        #[error("Timestamp Parse Error: {0}")]
        TimestampParse(#[from] TimestampError),
    }
}
