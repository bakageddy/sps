use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Deserialize, Serialize)]
pub struct DatabaseInfo {
    pub path: Option<String>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ParseReport {
    pub kinds: Vec<KindCount>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct KindCount {
    /// Outlines the path of the file we parsed.
    pub kind: String,
    /// Outlines the entries successfully parsed
    pub entries: u64,
    /// Outlines the errors application faced during parsing
    pub errors: u64,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DumpSummary {
    pub timestamp: u64,
    pub threads: u64,
    pub max_cpu: f32,
    pub total_cpu: f32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CPUPoint {
    pub cpu: f32,
    pub timestamp: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CPUThread {
    pub tid: u64,
    pub name: Option<String>,
    pub state: String,
    pub cpu: f32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CPUMemoryDumpSummary {
    pub timestamp: u64,
    pub total_cpu: f32,
    pub total_memory: f32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessUsage {
    pub pid: u64,
    pub name: String,
    pub user: Option<String>,
    pub value: f32,
    pub path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessSeries {
    pub cpu: Vec<CPUMemoryPoint>,
    pub memory: Vec<CPUMemoryPoint>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CPUMemoryPoint {
    pub timestamp: u64,
    pub value: f32,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AggregatedStuckthread {
    pub tid: u64,
    // NOTE: an begin: None means that the end stuckthread event only occurred without any begin
    pub begin: Option<u64>,
    pub end: Option<u64>, 
    pub name: String,
    pub duration: u64,
    pub request: Option<String>,
    pub active_start: Option<u64>,
    pub active_end: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MSSQLSnapshot {
    pub timestamp: u64,
    pub queries: u64,
    pub blocked: u64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockingSnapshot {
    pub timestamp: u64,
    pub chains: u64,
    pub sessions: u64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PGSQLSnapshot {
    pub timestamp: u64,
    pub queries: u64,
    pub waiting: u64,
    pub idle_in_txn: u64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MSSQLLongRunningQuery {
    pub session_id: u64,
    pub txn_id: u64,
    pub statement: String,
    pub login: String,
    pub snapshots: u64,
    pub first_seen: u64,
    pub last_seen: u64,
    pub max_elapsed: u64,
    pub max_cpu_time_ms: u64,
    pub blocked_in: u64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PGSQLLongRunningQuery {
    pub pid: u64,
    pub query: String,
    pub db_name: String,
    pub snapshots: u64,
    pub first_seen: u64,
    pub last_seen: u64,
    pub max_query_time_ms: Option<u64>,
    pub idle_in_txn_in: u64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MSSQLLongRunningTxn {
    pub session_id: u64,
    pub txn_id: u64,
    pub login: String,
    pub snapshots: u64,
    pub first_seen: u64,
    pub last_seen: u64,
    pub queries: Vec<String>,
}
