use super::types::{Agent, Collector, IngestConfig, Layout, TraceCount};
use crate::{ingest::types::{AgentLayoutQuery, AgentQuery, CountQuery}, types::{HttpClient, HttpError}};
use std::collections::HashMap;
use std::ops::Deref;

pub async fn connect(client: HttpClient) -> Result<Collector, HttpError> {
    let req = client.construct("/backend/layout", None::<HashMap<&str, &str>>)?;
    let res = client.send(req).await?;
    let collector = res.json::<Collector>().await?;
    Ok(collector)
}

pub async fn glowroot_agents(
    client: HttpClient,
    from: u64,
    to: u64,
    top_level_id: Option<String>,
) -> Result<Vec<Agent>, HttpError> {
    let path = if top_level_id.is_some() {
        "/backend/child-agent-rollups"
    } else {
        "/backend/top-level-agent-rollups"
    };
    let url = client.path(path)?;
    let params = AgentQuery {
        from,
        to,
        top_level_id,
    };

    let response = client.get(url).query(&params).send().await?;
    Ok(response.json().await?)
}

pub async fn agent_layout(client: HttpClient, id: String) -> Result<Layout, HttpError> {
    let url = client.path("/backend/agent-rollup-layout")?;
    let params = AgentLayoutQuery {
        agent_rollup_id: id,
    };
    let response = client.get(url).query(&params).send().await?;
    Ok(response.json().await?)
}

pub async fn count(client: HttpClient, config: &IngestConfig) -> Result<Vec<TraceCount>, HttpError> {
    let mut trace_counts = Vec::new();
    for tx_type in &config.transaction_types {
        let url = client.path("/backend/transaction/trace-count")?;

        let user = config.user.as_ref();
        let user_text = user.as_ref().map(|u| u.text.deref());
        let user_comparator = user.as_ref().map(|u| u.comparator.clone());

        let headline = config.headline.as_ref();
        let headline_text = headline.as_ref().map(|h| h.text.deref());
        let headline_comparator = headline.as_ref().map(|h| h.comparator.clone());

        let error = config.error_message.as_ref();
        let error_message = error.as_ref().map(|e| e.text.deref());
        let error_message_comparator = error.as_ref().map(|e| e.comparator.clone());

        let attr = config.attribute.as_ref();
        let attribute_name = attr.as_ref().map(|a| a.name.deref());
        let attribute_value = attr.as_ref().map(|a| a.value.text.deref());
        let attribute_value_comparator = attr.as_ref().map(|a| a.value.comparator.clone());

        let params = CountQuery {
            agent_rollup_id: &config.agent_rollup_id,
            transaction_type: &tx_type,
            transaction_name: config.transaction_name.as_deref(),
            from: config.from,
            to: config.to,
            headline: headline_text,
            headline_comparator,
            user: user_text,
            user_comparator,
            error_message,
            error_message_comparator,
            attribute_name,
            attribute_value,
            attribute_value_comparator,
        };

        let slow_count = client
            .get(url)
            .query(&params)
            .send()
            .await?
            .json::<u64>()
            .await?;


        let url = client.path("/backend/error/trace-count")?;
        let error_count = client
            .get(url)
            .query(&params)
            .send()
            .await?
            .json::<u64>()
            .await?;

        // NOTE: glowroot does not have more than 10 transaction_types in a
        // production setting, so the clone here is harmless compared to the number
        // of requests sent over the network
        trace_counts.push(TraceCount {
            transaction_type: tx_type.clone(),
            slow: slow_count,
            error: error_count,
        });
    }
    Ok(trace_counts)
}

pub async fn ingest(client: HttpClient, config: IngestConfig) -> Result<(), HttpError> {
    let url = client.path("/backend/transaction/points")?;

    todo!()
}
