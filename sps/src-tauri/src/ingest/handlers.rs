use super::glowroot;
use super::types::Agent;
use super::types::Collector;
use crate::{
    ingest::types::{IngestConfig, Layout, TraceCount},
    types::{AppState, HttpClient},
};
use reqwest::Url;
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
pub async fn glowroot_connect(
    url: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<Collector, String> {
    let url = Url::parse(&url).map_err(|e| format!("Failed to parse url: {e}"))?;
    let http_client =
        HttpClient::new(url).map_err(|e| format!("Failed to create http client: {e}"))?;
    state.lock().unwrap().client.replace(http_client.clone());

    glowroot::connect(http_client)
        .await
        .map_err(|e| format!("Failed to connect to glowroot: {e}"))
}

#[tauri::command]
pub async fn glowroot_agents(
    top_level_id: Option<String>,
    from: u64,
    to: u64,
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<Agent>, String> {
    let client = state.lock().unwrap().client.clone();
    if let Some(client) = client {
        glowroot::glowroot_agents(client, from, to, top_level_id)
            .await
            .map_err(|e| format!("Failed to fetch agents from glowroot: {e}"))
    } else {
        Err("Invalid state: Connect to glowroot first".to_owned())
    }
}

#[tauri::command]
pub async fn glowroot_agent_layout(
    agent_rollup_id: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<Layout, String> {
    let client = state.lock().unwrap().client.clone();
    if let Some(client) = client {
        glowroot::agent_layout(client, agent_rollup_id)
            .await
            .map_err(|e| format!("Failed to fetch agent layout from glowroot: {e}"))
    } else {
        Err("Invalid state: Connect to glowroot first".to_owned())
    }
}

#[tauri::command]
pub async fn glowroot_count(
    state: State<'_, Mutex<AppState>>,
    options: IngestConfig,
) -> Result<Vec<TraceCount>, String> {
    let client = state.lock().unwrap().client.clone();
    if let Some(client) = client {
        glowroot::count(client, &options)
            .await
            .map_err(|e| format!("Failed to fetch transaction type count from glowroot: {e}"))
    } else {
        Err("Invalid state: Connect to glowroot first".to_owned())
    }
}
