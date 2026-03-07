#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error with Stuck Threads: {0:?}")]
    StuckThread(#[from] stuckthread::Error),
    #[error("Error with Arguement parsing: {0:?}")]
    Clap(#[from] clap::Error),
    #[error("Error with SQLITE3: {0:?}")]
    Rusqlite(#[from] rusqlite::Error),
    #[error("Error during I/O: {0:?}")]
    IO(#[from] std::io::Error),
}

pub mod threaddump {
    #[derive(Debug, PartialEq, Eq, thiserror::Error)]
    pub enum Parse {
        #[error("Error parsing thread dump: Missing '@'")]
        MissingCommat,
        #[error("Error parsing object id expected {expected:?} characters, got {got:?}")]
        HexLen { expected: usize, got: usize },
        #[error("Error parsing object id: unexpected character: {got:?}")]
        HexUnexpectedChar { got: char },
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
        UnexpectedPreamble,
        #[error("Error during parsing thread state: Unexpected thread state")]
        UnexpectedThreadState,
        #[error("Error during parsing thread name: Double quotes not found")]
        DoubleQuoteNotFound,
        #[error("Error during parsing thread id: Equals not found")]
        EqualsNotFound,
        #[error("Error during parsing thread id: {0:?}")]
        ThreadIdParse(std::num::ParseIntError),
        #[error("Error during extracting thread id")]
        ThreadIDExtraction,
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
        #[error("Meta Data Parse Error: {0:?}")]
        MetaParseError(#[from] Meta),
        #[error("Meta Data Extraction Error")]
        MetaExtractionError,
    }

    #[derive(Debug, thiserror::Error)]
    pub enum Meta {
        #[error("Error during Meta Data parsing: `::` not found")]
        DoubleColonAbsent,
        #[error("Error during Meta Data parsing: {count:?}th ']' not found")]
        UnmatchedRightBracket {
            count: usize,
        },
        #[error("Error during Meta Data parsing: expected {expected:?} number of groups, got: {got:?}")]
        IncorrectHeaderInfoCount {
            got: Vec<String>,
            expected: usize,
        },

        #[error("Error during Meta Data parsing: expected {minimum_expected:?} number of groups, got: {got:?}")]
        IncorrectMessageInfoCount {
            got: Vec<String>,
            minimum_expected: usize,
        },

        #[error("Error: Invalid Date Format")]
        InvalidDateTimeFormat(#[from] time::error::Parse),

        #[error("Error: Invalid Date Format Description")]
        InvalidDateTimeFormatDescription(#[from] time::error::InvalidFormatDescription),

        #[error("Error during Meta Data parsing: invalid thread id, got: {got:?}, message: {inner:?}")]
        InvalidThreadId {
            got: String,
            inner: ParseIntError,
        },
        #[error("Error during Meta Data parsing: invalid active thread count, got: {got:?}, message: {inner:?}")]
        InvalidActiveThreadCount {
            got: String,
            inner: ParseIntError,
        },
        #[error("Error during Meta Data parsing: invalid active duration, got: {got:?}, message: {inner:?}")]
        InvalidActiveDuration {
            got: String,
            inner: ParseIntError,
        },

        #[error("Error during Meta Data parsing: Duration Overflow/Underflow")]
        DurationOverflow,
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
        #[error("Error during stacktrace element parsing: {0:?}")]
        ElementError(#[from] Element),
    }

    #[derive(Debug, thiserror::Error)]
    pub enum Element {
        #[error("Error during stacktrace element parsing: ( not found)")]
        OpenParenNotFound,
        #[error("Error during stacktrace element parsing: ) not found)")]
        CloseParenNotFound,
        #[error("Error during stacktrace element parsing: {0:?}")]
        SourceError(#[from] Source),
    }

    #[derive(Debug, thiserror::Error)]
    pub enum Source {
        #[error("Error during stacktrace element source parsing: Source type not recognized")]
        SourceTypeNotRecognized,
        #[error("Error during stacktrace element source parsing: Cannot parse line number")]
        LineNumber,
        #[error("Error during stacktrace element source parsing: Cannot find colon")]
        ColonNotFound,
    }
}
