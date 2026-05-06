use std::sync::{Arc, Mutex, OnceLock};

use crate::analysis::model::StuckThread;
use rmcp::{
    Json, ServerHandler, handler::server::wrapper::Parameters, schemars::JsonSchema, tool,
    tool_handler, tool_router,
};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
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
}
