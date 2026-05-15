#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error with Stuck Threads: {0:?}")]
    StuckThread(#[from] stuckthread::Error),
    #[error("Error with Thread Dumps: {0:?}")]
    ThreadDump(#[from] threaddump::Error),
    #[error("Error with Arguement parsing: {0:?}")]
    Clap(#[from] clap::Error),
    #[error("Error with SQLITE3: {0:?}")]
    Rusqlite(#[from] rusqlite::Error),
    #[error("Error with DuckDB: {0:?}")]
    DuckDB(#[from] duckdb::Error),
    #[error("Error during I/O: {0:?}")]
    IO(#[from] std::io::Error),
    #[error("Error with MCP communication/configuration/parameters: {0:?}")]
    MCP(#[from] mcp::Error),
}

pub mod mcp {
    #[derive(Debug, thiserror::Error, Eq, PartialEq)]
    pub enum Error {
        #[error("Error: Invalid params msg: {msg:?}")]
        InvalidParams { msg: String },
    }
}

pub mod scanner {
    use thiserror::Error;

    #[derive(Debug, Error, PartialEq, Eq)]
    pub enum Error {
        #[error("Error during consuming Scanner: Expected: {expected:?}, Got: {got:?}")]
        Expected { got: String, expected: String },
        #[error("Error during consuming Scanner: EndOfData")]
        EndOfData,
        #[error("Error during consuming Scanner: Delimiter {delimiter:?} not found on {data:?}")]
        DelimiterNotFound { delimiter: String, data: String },
        #[error(
            "Error during consuming Scanner: Trying to consume {expect:?} but available {have:?}"
        )]
        NotEnoughData { have: usize, expect: usize },
        #[error(
            "Error during consuming Scanner: {n:?}th byte is not a valid utf8 code point in {data:?}"
        )]
        NotACharBoundary { n: usize, data: String },
    }
}

pub mod threaddump {
    #[derive(Debug, PartialEq, Eq, thiserror::Error)]
    pub enum Error {
        #[error("Error during parsing thread dump: {0:?}")]
        Parse(#[from] Parse),
    }

    #[derive(Debug, PartialEq, Eq, thiserror::Error)]
    pub enum Parse {
        #[error("Error parsing thread dump: Missing '@'")]
        MissingCommat,
        #[error("Error parsing object id: unexpected input: {got:?}")]
        HexUnexpectedInput { got: String },
        #[error("Missing open paren")]
        OpenParenNotFound,
        #[error("Missing close paren")]
        CloseParenNotFound,
        #[error("Locked not found")]
        LockedNotFound,
        #[error("Colon (:) not found")]
        ColonNotFound,
        #[error("Error during parsing line number in function frame: {0:?}")]
        InvalidLineNumber(#[from] std::num::ParseIntError),
        #[error("Error during parsing thread state: Thread state preamble not found")]
        ExpectedPreamble,
        #[error("Error during parsing thread state: Unexpected thread state")]
        UnexpectedThreadState,
        #[error("Error during parsing thread name: Double quotes not found")]
        DoubleQuoteNotFound,
        #[error("Error during parsing thread id: Equals not found")]
        EqualsNotFound,
        #[error("Error during parsing thread id: {0:?}")]
        ThreadIdParse(std::num::ParseIntError),
        #[error("Error during extracting thread id")]
        ThreadIdExtraction,
        #[error("Error during extracting thread name")]
        ThreadNameExtraction,
        #[error("Error during extracting thread header")]
        ThreadHeaderExtraction,
        #[error("Error during extracting thread state")]
        ThreadStateExtraction,
        #[error("Error during extracting thread lock name")]
        ExpectedLockName,
        #[error("Error during extracting thread owner id")]
        ExpectedOwnerId,
        #[error("Error during extracting thread owner name")]
        ExpectedOwnerName,
        #[error("Error during extracting thread lock information")]
        ExpectedValidLockInformation,
        #[error("Error during parsing thread: Expected newline")]
        ExpectedNewline,
        #[error("Error during parsing thread dump: Expected \"Thread dump\"")]
        ThreadDumpExtraction,
        #[error("Error during parsing thread dump snapshot")]
        DumpSnapshot,
        #[error("Error during parsing thread dump: Snapshot Timestamp extraction")]
        SnapshotTimestampExtraction,
        #[error("Error during parsing thread dump: Snapshot parsing")]
        SnapshotTimestampParsing(#[from] time::error::Parse),
        #[error("Error during parsing thread dump: Snapshot Unix Time stamp conversion")]
        SnapshotTimestampConversion,
        #[error("Error during parsing thread state: Expected Wait Object")]
        ExpectedWaitObject,
        #[error("Error during parsing thread state: Expected Lock Object")]
        ExpectedLockObject,
    }
}

pub mod stuckthread {
    use std::num::ParseIntError;

    use crate::error::stacktrace;

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Error: {0:?}")]
        ParseError(#[from] Parse),

        #[error("Error: {0:?}")]
        NotValidUTF8(#[from] std::str::Utf8Error),
    }

    #[derive(Debug, thiserror::Error)]
    pub enum Parse {
        #[error("Stack Trace Parse Error: {0:?}")]
        StackParseError(#[from] stacktrace::Parse),
        #[error("Error during Meta Data parsing: `::` not found")]
        DoubleColonAbsent,
        #[error("Error during Meta Data parsing: {count:?}th ']' not found")]
        UnmatchedRightBracket { count: usize },
        #[error(
            "Error during Meta Data parsing: expected {expected:?} number of groups, got: {got:?}"
        )]
        IncorrectHeaderInfoCount { got: Vec<String>, expected: usize },

        #[error(
            "Error during Meta Data parsing: expected {minimum_expected:?} number of groups, got: {got:?}"
        )]
        IncorrectMessageInfoCount {
            got: Vec<String>,
            minimum_expected: usize,
        },

        #[error("Error: Invalid Date Format")]
        InvalidDateTimeFormat(#[from] time::error::Parse),

        #[error("Error: Invalid Date Format Description")]
        InvalidDateTimeFormatDescription(#[from] time::error::InvalidFormatDescription),

        #[error(
            "Error during Meta Data parsing: invalid thread id, got: {got:?}, message: {inner:?}"
        )]
        InvalidThreadId { got: String, inner: ParseIntError },
        #[error(
            "Error during Meta Data parsing: invalid active thread count, got: {got:?}, message: {inner:?}"
        )]
        InvalidActiveThreadCount { got: String, inner: ParseIntError },
        #[error(
            "Error during Meta Data parsing: invalid active duration, got: {got:?}, message: {inner:?}"
        )]
        InvalidActiveDuration { got: String, inner: ParseIntError },

        #[error("Error during Meta Data parsing: Duration Overflow/Underflow")]
        DurationOverflow,
        #[error("Meta Data Extraction Error")]
        MetaExtractionError,
    }
}

pub mod stacktrace {
    // use std::path::PathBuf;
    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        // ParseError { line: usize, file: PathBuf, inner: Parse },
        #[error("Error during stacktrace parsing: {0:?}")]
        ParseError(#[from] Parse),
    }

    #[derive(Debug, thiserror::Error)]
    pub enum Parse {
        #[error("Error during stacktrace element parsing: Throwable not found")]
        ThrowableNotFound,
        #[error("Error during stacktrace element parsing: at not found")]
        AtNotFound,
        #[error("Error during stacktrace element parsing: parenthesis not found")]
        ParenNotFound,
        #[error("Error during stacktrace element parsing: ) not found)")]
        CloseParenNotFound,
        #[error("Error during stacktrace element source parsing: Source type not recognized")]
        SourceTypeNotRecognized,
        #[error("Error during stacktrace element source parsing: Cannot parse line number")]
        LineNumber,
        #[error("Error during stacktrace element source parsing: Cannot find colon")]
        ColonNotFound,
    }
}
