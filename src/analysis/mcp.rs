use std::sync::{Arc, Mutex, OnceLock};

use crate::analysis::model::{Frame, StuckThread};
use rmcp::{
    Json, ServerHandler, handler::server::wrapper::Parameters, schemars::JsonSchema, tool,
    tool_handler, tool_router,
};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use time::UtcDateTime;
use tokio::task::spawn_blocking;
use tracing::debug;

pub static DB: OnceLock<Arc<Mutex<Connection>>> = OnceLock::new();

pub fn init_db(cnx: Connection) {
    let cnx = Arc::new(Mutex::new(cnx));
    DB.set(cnx).expect("Failed to set DB connection pool")
}

pub struct AnalysisServer {
    state: Arc<Mutex<Connection>>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct StuckThreadsRange {
    start: i64,
    end: i64,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum StuckThreadResponseInner {
    Error { why: String, got: Option<String> },
    Success { threads: Vec<StuckThread> },
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct StuckThreadResponse {
    #[serde(flatten)]
    inner: StuckThreadResponseInner,
}

impl StuckThreadResponse {
    pub fn success(threads: Vec<StuckThread>) -> Self {
        Self {
            inner: StuckThreadResponseInner::Success { threads },
        }
    }

    pub fn error(why: String, got: Option<String>) -> Self {
        Self {
            inner: StuckThreadResponseInner::Error { why, got },
        }
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct StuckThreadSummary {
    #[serde(flatten)]
    inner: StuckThreadSummaryInner,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum StuckThreadSummaryInner {
    Success {
        first_seen_unix_ms: i64,
        last_seen_unix_ms: i64,
        first_seen_utc: String,
        last_seen_utc: String,
        most_frequent_by_name: FrequentThreadByName,
        longest_thread: LongestRunningThread,
        count: i64,
    },
    Error {
        reason: String,
    },
}

impl StuckThreadSummary {
    pub fn sucess(
        first_seen_unix: i64,
        last_seen_unix: i64,
        most_freq: FrequentThreadByName,
        longest: LongestRunningThread,
        count: i64,
    ) -> Self {
        Self {
            inner: StuckThreadSummaryInner::Success {
                first_seen_unix_ms: first_seen_unix,
                last_seen_unix_ms: last_seen_unix,
                first_seen_utc: UtcDateTime::from_unix_timestamp(first_seen_unix / 1000)
                    .unwrap_or(UtcDateTime::UNIX_EPOCH)
                    .to_string(),
                last_seen_utc: UtcDateTime::from_unix_timestamp(last_seen_unix / 1000)
                    .unwrap_or(UtcDateTime::UNIX_EPOCH)
                    .to_string(),
                most_frequent_by_name: most_freq,
                longest_thread: longest,
                count,
            },
        }
    }

    pub fn error(reason: String) -> Self {
        Self {
            inner: StuckThreadSummaryInner::Error { reason },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FrequentThreadByName {
    name: Option<String>,
    count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LongestRunningThread {
    trace_peek: Vec<Frame>,
    name: Option<String>,
    request: Option<String>,
    start_utc: String,
    end_utc: String,
    start_unix_ms: i64,
    duration_ms: i64,
    thread_id: i64,
    trace_id: i64,
}

#[tool_handler]
impl ServerHandler for AnalysisServer {}

#[tool_router()]
impl AnalysisServer {
    pub fn new() -> Self {
        let cnx = DB.get().expect("init_db must be called before MCP server");
        Self { state: cnx.clone() }
    }

    #[tool(
        name = "get_stuckthreads_between_range",
        description = "Fetches all the stuckthreads between the range `start` and `end`. `start` & `end` must be a UNIX timestamp in UTC from epoch in milliseconds, not seconds"
    )]
    pub async fn get_stuckthreads_between_range(
        &self,
        Parameters(params): Parameters<StuckThreadsRange>,
    ) -> Json<StuckThreadResponse> {
        debug!("get_stuckthreads_between_range invoked with {params:?}");
        if params.start > params.end {
            return Json(StuckThreadResponse::error(
                "`start` should be lesser than or equal to `end`".to_string(),
                Some(format!("start: {}, end: {}", params.start, params.end)),
            ));
        }

        let cnx = self.state.clone();
        let output = spawn_blocking(move || {
            let cnx = match cnx.lock() {
                Ok(cnx) => cnx,
                Err(e) => e.into_inner(),
            };
            StuckThread::get_by_range(&cnx, params.start, params.end)
        })
        .await;

        match output {
            Ok(Ok(threads)) => Json(StuckThreadResponse::success(threads)),
            Ok(Err(e)) => Json(StuckThreadResponse::error(e.to_string(), None)),
            Err(e) => Json(StuckThreadResponse::error(e.to_string(), None)),
        }
    }

    #[tool(
        name = "get_stuckthread_summary",
        description = "Fetches an summary of the stuckthreads that outlines the first and last seen stuckthread, stuckthread with the most frequent name, stuckthread that's running the longest and the total number of stuckthreads present in the database"
    )]
    pub async fn get_stuckthread_summary(&self) -> Json<StuckThreadSummary> {
        debug!("get_stuckthread_summary invoked");
        let cnx = self.state.clone();
        let (first_seen_unix, last_seen_unix, summary_count) = match spawn_blocking(move || {
            let cnx = match cnx.lock() {
                Ok(cnx) => cnx,
                Err(e) => e.into_inner(),
            };

            StuckThread::get_stuckthread_summary(&cnx)
        })
        .await
        {
            Ok(Ok(v)) => v,
            Ok(Err(v)) => return Json(StuckThreadSummary::error(v.to_string())),
            Err(e) => return Json(StuckThreadSummary::error(e.to_string())),
        };

        let cnx = self.state.clone();
        let (name, freq_count) = match spawn_blocking(move || {
            let cnx = match cnx.lock() {
                Ok(cnx) => cnx,
                Err(e) => e.into_inner(),
            };
            StuckThread::get_most_frequent_by_name(&cnx)
        })
        .await
        {
            Ok(Ok(v)) => v,
            Ok(Err(v)) => return Json(StuckThreadSummary::error(v.to_string())),
            Err(e) => return Json(StuckThreadSummary::error(e.to_string())),
        };
        let most_freq = FrequentThreadByName {
            name: name,
            count: freq_count,
        };

        let cnx = self.state.clone();
        let (peek, start_utc, end_utc, name, request, start, active_ms, thread_id, stack_id) =
            match spawn_blocking(move || {
                let cnx = match cnx.lock() {
                    Ok(cnx) => cnx,
                    Err(e) => e.into_inner(),
                };

                StuckThread::get_longest_stuck_thread(&cnx)
            })
            .await
            {
                Ok(Ok(v)) => v,
                Ok(Err(v)) => return Json(StuckThreadSummary::error(v.to_string())),
                Err(e) => return Json(StuckThreadSummary::error(e.to_string())),
            };

        let longest_thread = LongestRunningThread {
            trace_peek: peek,
            start_utc,
            end_utc,
            name,
            request,
            start_unix_ms: start,
            duration_ms: active_ms,
            thread_id,
            trace_id: stack_id,
        };

        debug!(
            "{:?}",
            StuckThreadSummary::sucess(
                first_seen_unix,
                last_seen_unix,
                most_freq.clone(),
                longest_thread.clone(),
                summary_count
            )
        );
        Json(StuckThreadSummary::sucess(
            first_seen_unix,
            last_seen_unix,
            most_freq,
            longest_thread,
            summary_count,
        ))
    }
}
