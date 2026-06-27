use std::net::Ipv4Addr;

use memchr::memmem::find;

#[derive(Debug)]
pub enum Strategy {
    MSSQL,
    PGSQL,
}

impl Strategy {
    fn contains(haystack: &[u8], needle: &[u8]) -> bool {
        find(haystack, needle).is_some()
    }
    pub fn detect(body: &[u8]) -> Option<Self> {
        if Self::contains(body, b"pid") || Self::contains(body, b"Query Time (s)") {
            Some(Self::PGSQL)
        } else if Self::contains(body, b"Logical Reads")
            || Self::contains(body, b"Wait Resource")
            || Self::contains(body, b"CPUTime")
        {
            Some(Self::MSSQL)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct PGSQLRunningQueries<'a> {
    pub pid: u64,
    pub query_time_ms: u64,
    pub txn_time_ms: u64,
    pub db_name: &'a str,
    pub state: State<'a>,
    pub waiting: bool,
    pub query: &'a str,
    pub last_state_change: u64,
    pub application_name: &'a str,
    pub client_address: Option<Ipv4Addr>,
    pub client_port: Option<u16>,
    pub client_hostname: Option<&'a str>
}

#[derive(Debug)]
pub enum State<'a> {
    Active,
    Idle,
    Unknown(&'a str)
}

#[derive(Debug)]
pub enum MSSQLRunningQueryTable<'a> {
    SPWho2(SPWho2Table<'a>),
    RunningQueries(MSSQLRunningQueries<'a>),
}

#[derive(Debug)]
pub struct SPWho2Table<'a> {
    pub spid: u64,
    pub status: SPWho2Status<'a>,
    pub login: &'a str,
    pub hostname: &'a str,
    pub blocked_by: Option<u64>,
    pub db_name: &'a str,
    pub command: &'a str,
    pub cpu_time: u64,
    pub disk_io: u64,
    pub last_batch: &'a str,
    pub program_name: &'a str,
    pub request_id: &'a str,
}

#[derive(Debug)]
pub enum SPWho2Status<'a> {
    Sleeping,
    Background,
    Runnable,
    Unknown(&'a str)
}

#[derive(Debug)]
pub struct MSSQLRunningQueries<'a> {
    pub session_id: u64,
    pub status: Status,
    pub txn_id: u64,
    pub blocked_by: u64,
    pub wait_type: Option<&'a str>,
    pub wait_resource: Option<&'a str>,
    pub wait_time_ms: u64,
    pub cpu_time_ms: u64,
    pub logical_reads: u64,
    pub physical_reads: u64,
    pub physical_writes: u64,
    pub elapsed_time_ms: u64,
    pub statement: &'a str,
    pub command_text: Option<&'a str>,
    pub command: Option<&'a str>,
    pub login_name: &'a str,
    pub host_name: &'a str,
    pub db_name: &'a str,
    pub program_name: &'a str,
    pub host_process_id: u64,
    pub last_request_end_ms: u64,
    pub login_time_ms: u64,
    pub open_transaction_count: u64,
}

#[derive(Debug)]
pub enum Status {
    Running,
    Runnable,
    Suspended,
}
