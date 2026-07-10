#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error with Stuck Threads: {0}")]
    StuckThread(#[from] stuckthread::Error),
    #[error("Error with Thread Dumps: {0}")]
    ThreadDump(#[from] threaddump::Error),
    #[error("Error with Stuck Queries: {0}")]
    StuckQuery(#[from] stuckquery::Error),
    #[error("Error with Stuck Queries: {0}")]
    RunningQuery(#[from] running_query::Error),
    #[error("Error with Detecting OS/DB")]
    Detection,
    #[error("Error with Arguement parsing: {0}")]
    Clap(#[from] clap::Error),
    #[error("Error with DuckDB: {0}")]
    DuckDB(#[from] duckdb::Error),
    #[error("Error with R2D2 Connection Pool: {0}")]
    R2D2(#[from] r2d2::Error),
    #[error("Error during I/O: {0}")]
    IO(#[from] std::io::Error),
    #[error("Error with MCP communication/configuration/parameters: {0}")]
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
    use std::str::Utf8Error;

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
        #[error("Error during conversion from bytes to utf8: {0}")]
        UTF8(#[from] Utf8Error),
    }
}

pub mod stuckquery {
    use std::{
        net::AddrParseError,
        num::{ParseFloatError, ParseIntError},
    };

    use crate::error::scanner;

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Error during StuckQuery kind detection: Unable to detect kind")]
        UnableToDetectKind,
        #[error("Error during parsing pgsql stuckqueries: {0:?}")]
        PGParse(#[from] PGParse),
        #[error("Error during parsing SQL Server stuckqueries: {0:?}")]
        MSSQLParse(#[from] MSSQLParse),
    }

    #[derive(Debug, thiserror::Error)]
    pub enum MSSQLParse {
        #[error("Empty Iterator on SQL Server Stuck Query table")]
        EmptyBlock,
        #[error("Error extracting stuckquery table header")]
        TableExtraction,
        #[error("Error extracting stuckquery table header")]
        TableHeaderExtraction,
        #[error("Error during timestamp/meta information extraction from mssql")]
        TableHeaderMetaExtraction,
        #[error("Error during parsing timestamp information from the mssql table header: {0}")]
        TimestampParse(time::error::Parse),

        #[error("Error during extracting session id information from the table: {0}")]
        SessionIDExtraction(scanner::Error),
        #[error("Error during parsing session id: {0}")]
        SessionIDParse(ParseIntError),
        #[error("Error during status extraction from the table: {0}")]
        StatusExtraction(scanner::Error),
        #[error("Error during parsing status information, Unrecognized status: {0}")]
        InvalidStatus(String),
        #[error("Error during extracting transaction id information from the table: {0}")]
        TransactionIDExtraction(scanner::Error),
        #[error("Error during parsing transaction id: {0}")]
        TransactionIDParse(ParseIntError),
        #[error("Error during extracting blocked by information from the table: {0}")]
        BlockedByExtraction(scanner::Error),
        #[error("Error during parsing blocked by column: {0:?}")]
        BlockedByParse(ParseIntError),
        #[error("Error during extracting wait type information from the table: {0}")]
        WaitTypeExtraction(scanner::Error),
        #[error("Error during extracting wait resource information from the table: {0}")]
        WaitResourceExtraction(scanner::Error),
        #[error("Error during extracting wait time information from the table: {0}")]
        WaitTimeExtraction(scanner::Error),
        #[error("Error during parsing wait time information from the table: {0}")]
        WaitTimeParse(ParseFloatError),
        #[error("Error during extracting CPU time information from the table: {0}")]
        CPUTimeExtraction(scanner::Error),
        #[error("Error during parsing CPU time information from the table: {0}")]
        CPUTimeParse(ParseFloatError),
        #[error("Error during extracting Logical reads information from the table: {0}")]
        LogicalReadsExtraction(scanner::Error),
        #[error("Error during parsing Logical reads information from the table: {0}")]
        LogicalReadsParse(ParseIntError),
        #[error("Error during extracting Physical Reads information from the table: {0}")]
        PhysicalReadsExtraction(scanner::Error),
        #[error("Error during parsing Physical reads information from the table: {0}")]
        PhysicalReadsParse(ParseIntError),
        #[error("Error during extracting Physical Writes information from the table: {0}")]
        PhysicalWritesExtraction(scanner::Error),
        #[error("Error during parsing Physical Writes information from the table: {0}")]
        PhysicalWritesParse(ParseIntError),
        #[error("Error during extracting Elapsed Time information from the table: {0}")]
        ElapsedTimeExtraction(scanner::Error),
        #[error("Error during parsing Elapsed Time information from the table: {0}")]
        ElapsedTimeParse(ParseFloatError),
        #[error("Error during extracting Statement information from the table: {0}")]
        StatementExtraction(scanner::Error),
        #[error("Error during extracting Command Text information from the table: {0}")]
        CommandTextExtraction(scanner::Error),
        #[error("Error during extracting Command information from the table: {0}")]
        CommandExtraction(scanner::Error),
        #[error("Error during extracting Login Name information from the table: {0}")]
        LoginNameExtraction(scanner::Error),
        #[error("Error during extracting Host Name information from the table: {0}")]
        HostNameExtraction(scanner::Error),
        #[error("Error during extracting Database Name information from the table: {0}")]
        DatabaseNameExtraction(scanner::Error),
        #[error("Error during extracting Program Name information from the table: {0}")]
        ProgramNameExtraction(scanner::Error),
        #[error("Error during extracting Host Process ID information from the table: {0}")]
        HostProcessIDExtraction(scanner::Error),
        #[error("Error during parsing Host Process ID information from the table: {0}")]
        HostProcessIDParse(ParseIntError),
        #[error("Error during extracting Last Request End information from the table: {0}")]
        LastRequestEndExtraction(scanner::Error),
        #[error("Error during parsing Last Request End information from the table: {0}")]
        LastRequestEndParse(time::error::Parse),
        #[error("Error during extracting Login Time information from the table: {0}")]
        LoginTimeExtraction(scanner::Error),
        #[error("Error during parsing Last Login Time information from the table: {0}")]
        LoginTimeParse(time::error::Parse),
        #[error("Error during extracting Open Transaction ID information from the table: {0}")]
        OpenTransactionCountExtraction(scanner::Error),
        #[error("Error during parsing Open Transaction ID information from the table: {0}")]
        OpenTransactionCountParse(ParseIntError),
    }

    #[derive(Debug, thiserror::Error)]
    pub enum PGParse {
        #[error("Error during parsing Stuck Query State: {0:?}")]
        UnrecognizedState(String),
        #[error("Empty Iterator on Stuck query table")]
        EmptyBlock,
        #[error("Error during parsing client address: {0}")]
        AddrParse(#[from] AddrParseError),
        #[error("Error during parsing process id: {0}")]
        PidParse(ParseIntError),
        #[error("Error extracting PID from the table: {0}")]
        PidExtraction(scanner::Error),
        #[error("Error parsing PID from the table: {0}")]
        InvalidPID(ParseIntError),
        #[error("Error extracting Query Time information from the table: {0}")]
        QueryTimeExtraction(scanner::Error),
        #[error("Error parsing Query Time information from the table: {0}")]
        InvalidQueryTime(ParseFloatError),
        #[error("Error extracting Transaction Time information from the table")]
        TransactionTimeExtraction(scanner::Error),
        #[error("Error parsing Transaction Time information from the table: {0}")]
        InvalidTransactionTime(ParseFloatError),
        #[error("Error extracting Database Name information from the table: {0}")]
        DatabaseNameExtraction(scanner::Error),
        #[error("Error extracting State information from the table: {0}")]
        StateExtraction(scanner::Error),
        #[error("Error extracting Waiting information from the table: {0}")]
        WaitingExtraction(scanner::Error),
        #[error("Invalid Waiting state information from the table, got: {got:?}, expected: t/f")]
        InvalidWaitingState { got: String },
        #[error("Error extracting Query information from the table: {0}")]
        QueryExtraction(scanner::Error),
        #[error("Error extracting State Change information from the table: {0}")]
        StateChangeExtraction(scanner::Error),
        #[error("Error parsing State Change information from the table: {0}")]
        InvalidStateChange(time::error::Parse),
        #[error("Error extracting Application Name information from the table: {0}")]
        ApplicationNameExtraction(scanner::Error),
        #[error("Error extracting Client Address information from the table: {0}")]
        ClientAddressExtraction(scanner::Error),
        #[error("Error extracting Client Hostname information from the table: {0}")]
        ClientHostnameExtraction(scanner::Error),
        #[error("Error extracting Client Port information from the table: {0}")]
        ClientPortExtraction(scanner::Error),
        #[error("Error parsing Client Port information from the table: {0}")]
        InvalidClientPort(ParseIntError),
        #[error("Error parsing stuckquery table. Expected 8 lines of information")]
        TableExtraction,
        #[error("Error extracting stuckquery table header")]
        TableHeaderExtraction,
        #[error("Error extracting timestamp/meta information from the table header line")]
        TableHeaderMetaExtraction,
        #[error("Error during parsing timestamp information from the table header: {0}")]
        TimestampParse(time::error::Parse),
    }
}

pub mod running_query {
    use std::{
        net::AddrParseError,
        num::{ParseFloatError, ParseIntError},
    };

    use crate::error::scanner;

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Error during parsing PGSQL Running Queries: {0}")]
        PGParse(#[from] PGParse),
        #[error("Error during parsing MSSQL Running Queries: {0}")]
        MSParse(#[from] MSParse),
    }

    #[derive(Debug, thiserror::Error)]
    pub enum PGParse {
        #[error("Error during Extracting PGSQL Running Query PID: {0}")]
        PIDExtraction(scanner::Error),
        #[error("Error during Parsing PGSQL Running Query PID: {0}")]
        PIDParse(ParseIntError),
        #[error("Error during Extracting PGSQL Running Query, Query Time: {0}")]
        QueryTimeExtraction(scanner::Error),
        #[error("Error during Parsing PGSQL Running Query, Query Time: {0}")]
        QueryTimeParse(ParseFloatError),
        #[error("Error during Extracting PGSQL Running Query, Transaction Time: {0}")]
        TransactionTimeExtraction(scanner::Error),
        #[error("Error during Parsing PGSQL Running Query, Transaction Time: {0}")]
        TransactionTimeParse(ParseFloatError),
        #[error("Error during Extracting PGSQL Running Query, Database Name: {0}")]
        DatabaseNameExtraction(scanner::Error),
        #[error("Error during Extracting PGSQL Running Query, State Information: {0}")]
        StateExtraction(scanner::Error),
        #[error("Error during Extracting PGSQL Running Query: Waiting State: {0}")]
        WaitingExtraction(scanner::Error),
        #[error("Error during Parsing PGSQL Running Query: Waiting State: {got}")]
        InvalidWaitingState {
            got: String,
        },
        #[error("Error during Parsing PGSQL Running Query, `Query` {0}")]
        QueryExtraction(scanner::Error),
        #[error("Error during Parsing PGSQL Running Query, `Query` {0}")]
        ClientAddressParsing(AddrParseError),
        #[error("Error during Extracting PGSQL Running Query, `Client Address` {0}")]
        ClientAddressExtraction(scanner::Error),
        #[error("Error during Extracting PGSQL Running Query, `Application Name` {0}")]
        ApplicationNameExtraction(scanner::Error),
        #[error("Error during Parsing PGSQL Running Query, `Last State Change` {0}")]
        LastStateChangeParse(#[from] time::error::Parse),
        #[error("Error during Extracting PGSQL Running Query, `Last State Change` {0}")]
        StateChangeExtraction(scanner::Error),
        #[error("Error during Extracting PGSQL Running Query, `Last State Change` {0}")]
        ClientPortExtraction(scanner::Error),
        #[error("Error during Parsing PGSQL Running Query, `Client Port` {0}")]
        ClientPortParsing(ParseIntError),
        #[error("Error during Extraction PGSQL Running Query, `Client Host` {0}")]
        ClientHostExtraction(scanner::Error),
    }

    #[derive(Debug, thiserror::Error)]
    pub enum MSParse {}
}

pub mod cpumonitoring {
    use crate::error::{scanner, stacktrace};
    use std::num::{ParseFloatError, ParseIntError};

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Error during parsing CPU Monitoring Logs: {0:?}")]
        Parse(#[from] Parse),
    }

    #[derive(Debug, thiserror::Error)]
    pub enum Parse {
        #[error("Error during parsing state: UnrecognizedState {0:?}")]
        UnrecognizedState(String),

        #[error("Error during extracting Time/Date from header")]
        MonitoringHeaderExtraction,
        #[error("Error during parsing Time/Date from header: {0:?}")]
        MonitoringHeaderParse(time::error::Parse),
        #[error("Error during extracting Thread Info")]
        MonitoringThreadInfoExtraction(scanner::Error),
        #[error("Error during extracting Thread Id")]
        MonitoringThreadIdExtraction(scanner::Error),
        #[error("Error during extracting Thread Id")]
        MonitoringThreadIdCommaExtraction,
        #[error("Error during Parsing Thread Id: {0:?}")]
        MonitoringThreadIdParse(#[from] ParseIntError),
        #[error("Error during extracting Thread CPU Usage: {0:?}")]
        MonitoringThreadCPUUsageExtraction(scanner::Error),
        #[error("Error during extracting Thread CPU Usage: Incomplete Line")]
        MonitoringThreadCPUUsageIncompleteLine,
        #[error("Error during Parsing Thread CPU Usage: {0:?}")]
        MonitoringThreadCPUParse(#[from] ParseFloatError),
        #[error("Error during extracting Thread Name: {0:?}")]
        MonitoringThreadNameExtraction(scanner::Error),
        #[error("Error during extracting Thread Name")]
        MonitoringThreadNameIncompleteLine,
        #[error("Error during extracting Thread State: {0:?}")]
        MonitoringThreadStateExtraction(scanner::Error),
        #[error("Error during extracting Thread State Incomplete Line")]
        MonitoringThreadStateIncompleteLine,

        #[error("Error during parsing stack trace: {0:?}")]
        MonitoringTraceParse(#[from] stacktrace::Parse),
    }
}

pub mod threaddump {
    use crate::error::stacktrace;

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Error during parsing thread dump: {0:?}")]
        Parse(#[from] Parse),
    }

    #[derive(Debug, thiserror::Error)]
    pub enum Parse {
        #[error("Error parsing thread dump: Missing '@'")]
        MissingCommat,
        #[error("Missing open paren")]
        OpenParenNotFound,
        #[error("Missing close paren")]
        CloseParenNotFound,
        #[error("Locked not found")]
        LockedNotFound,
        #[error("Colon (:) not found")]
        ColonNotFound,
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
        #[error("Error during parsing stacktrace/object related data: {0:?}")]
        TraceParse(#[from] stacktrace::Parse),
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
        #[error("Error during Meta Data parsing: invalid active thread count, got: {got:?}")]
        InvalidActiveThreadCount { got: String },
        #[error("Error during Meta Data parsing: invalid active duration, got: {got:?}")]
        InvalidActiveDuration { got: String },

        #[error("Error during Meta Data parsing: Duration Overflow/Underflow")]
        DurationOverflow,
        #[error("Meta Data Extraction Error")]
        MetaExtractionError,
    }
}

pub mod stacktrace {
    use std::str::Utf8Error;

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
        #[error("Error during stacktrace element source parsing: Cannot find @")]
        MissingCommat,
        #[error("Error during stacktrace element source parsing: Cannot convert to UTF8 {0:?}")]
        UTF8Conversion(#[from] Utf8Error),
        #[error("Error parsing object id: unexpected input: {got:?}")]
        InvalidHexadecimalCharacter { got: String },
        #[error("Error during parsing line number in function frame: {0:?}")]
        InvalidLineNumber(#[from] std::num::ParseIntError),

        #[error("Empty element during stacktrace parsing")]
        EmptyElement,
    }
}

pub mod cpumemstats {
    use std::num::{ParseFloatError, ParseIntError};

    use crate::error::scanner;

    #[derive(thiserror::Error, Debug)]
    pub enum Error {
        #[error("Error during parsing: {0:?}")]
        Parse(#[from] Parse),
    }

    #[derive(thiserror::Error, Debug)]
    pub enum Parse {
        #[error("Error during name extraction: {0}")]
        NameExtraction(scanner::Error),
        #[error("Error during PID extraction: {0}")]
        PIDExtraction(scanner::Error),
        #[error("Error during parsing PID: {0}")]
        PIDParse(ParseIntError),
        #[error("Error during CPU Usage extraction: {0}")]
        CPUUsageExtraction(scanner::Error),
        #[error("Error during parsing CPU Usage : {0}")]
        CPUParse(ParseFloatError),
        #[error("Error during CPU Usage extraction: {0}")]
        PathExtraction(scanner::Error),
        #[error("Error during CPU Usage extraction: {0}")]
        MemoryUsageExtraction(scanner::Error),
        #[error("Error during parsing CPU Usage : {0}")]
        MemoryParse(ParseFloatError),
        #[error("Error during timestamp extraction: {0}")]
        TimestampExtraction(scanner::Error),
        #[error("Error during timestamp parsing: {0}")]
        TimestampParsing(#[from] time::error::Parse),
        #[error("Error during header extraction")]
        HeaderExtraction,
        #[error("Error during log type extraction: {0}")]
        LogTypeExtraction(scanner::Error),
        #[error("Invalid Log Type found")]
        InvalidLogType,
        #[error("Error during total cpu extraction: {0}")]
        TotalUsageExtraction(scanner::Error),
        #[error("Error during Total CPU/Memory Usage: {0}")]
        TotalUsageParsing(ParseFloatError),
        #[error("Total Usage unavailable")]
        TotalUsageUnavailable,
    }
}
