use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Collector {
    pub url: Option<String>,
    pub version: String,
    pub central: bool,
    pub login_enabled: bool,
    pub logged_in: bool,
    #[serde(rename = "redirectToLogin")]
    pub needs_login: bool,
    pub default_time_zone_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Agent {
    pub id: String,
    pub display: String,
    pub disabled: bool,
    // pub rollup: bool,
    // pub has_children: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Layout {
    pub transaction_types: Vec<String>,
    // pub trace_attribute_names: HashMap<String, Vec<String>>,
    pub permissions: Permission,
    pub default_percentiles: Vec<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Permission {
    pub transaction: TransactionPermission,
    pub error: ErrorPermission,
    // TODO: If you need to manually invoke a threaddump/heapdump in the hostname
    // implement this feature
    // pub jvm: JVMPermission,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TransactionPermission {
    pub overview: bool,
    pub traces: bool,
    pub queries: bool,
    pub thread_profile: bool,
    pub service_calls: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ErrorPermission {
    pub overview: bool,
    pub traces: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IngestConfig {
    pub agent_rollup_id: String,
    pub transaction_types: Vec<String>,
    pub transaction_name: Option<String>,
    pub slow: bool,
    pub error: bool,
    pub from: u64,
    pub to: u64,
    pub duration_millis_low: u64,
    pub duration_millis_high: Option<u64>,
    pub max_traces: u64,
    pub workers: u64,
    pub entries: bool,
    pub queries: bool,
    pub profiles: bool,
    pub skip_existing: bool,
    pub headline: Option<TextFilter>,
    pub user: Option<TextFilter>,
    pub error_message: Option<TextFilter>,
    pub attribute: Option<Attribute>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Attribute {
    pub name: String,
    pub value: TextFilter,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TextFilter {
    pub text: String,
    pub comparator: Comparator,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum Comparator {
    Begins,
    Equals,
    Ends,
    Contains,
    NotContains,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct CountQuery<'a> {
    pub agent_rollup_id: &'a str,
    pub transaction_type: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_name: Option<&'a str>,
    pub from: u64,
    pub to: u64,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub headline: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headline_comparator: Option<Comparator>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<&'a str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_comparator: Option<Comparator>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<&'a str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message_comparator: Option<Comparator>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribute_name: Option<&'a str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribute_value: Option<&'a str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribute_value_comparator: Option<Comparator>,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct AgentQuery {
    pub from: u64,
    pub to: u64,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_level_id: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct AgentLayoutQuery {
    pub agent_rollup_id: String,
}

impl Comparator {
    pub fn to_string(&self) -> String {
        match self {
            Self::Begins => String::from("begins"),
            Self::Ends => String::from("ends"),
            Self::Equals => String::from("equals"),
            Self::Contains => String::from("contains"),
            Self::NotContains => String::from("not-contains"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TraceCount {
    pub transaction_type: String,
    pub slow: u64,
    pub error: u64,
}

#[derive(Deserialize, Debug)]
pub struct TraceSummary {
    pub normal_points: Vec<TracePoint>,
    pub error_points: Vec<TracePoint>,
    pub partial_points: Vec<TracePoint>,
    pub limit_exceeded: bool,
    pub expired: bool,
}

#[derive(Deserialize, Debug)]
pub struct TracePoint(pub u64, pub f64, pub String, pub String);

impl TracePoint {
    pub fn get_capture_time(&self) -> u64 {
        self.0
    }

    pub fn get_duration_millis(&self) -> f64 {
        self.1
    }

    pub fn get_agent_id(&self) -> &str {
        &self.2
    }

    pub fn get_trace_id(&self) -> &str {
        &self.3
    }
}

