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
pub struct AggregatedStuckthreadMinimal {
    pub timestamp: u64,
    pub tid: u64,
    pub duration_ms: u64,
}
