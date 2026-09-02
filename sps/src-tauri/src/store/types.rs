use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Frame {
    pub method: String,
    pub source: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CPUMonitoring {
    pub cpu: f32,
    pub tid: u64,
    pub timestamp: u64,
    pub name: Option<String>,
    pub trace: Option<Vec<Frame>>,
}

