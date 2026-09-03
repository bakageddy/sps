use std::{borrow::Cow, collections::HashMap};

use duckdb::Connection;

use crate::{
    handlers::types::AggregatedStuckthread,
    parser::stuckthread::Frame,
    store::{error::Error, tables::Tables},
};

pub fn get_stuckthread_aggregates(
    cnx: &Connection,
    from: Option<u64>,
    to: Option<u64>,
) -> Result<Vec<AggregatedStuckthread>, Error> {
    let query = format!(
        "SELECT {0}.timestamp, {0}.tid, {0}.duration, {0}.name, {0}.request, {0}.active FROM {0} WHERE timestamp BETWEEN $1 AND $2 ORDER BY timestamp",
        Tables::Stuckthread.into_str()
    );

    let from = from.unwrap_or(0);
    let to = to.unwrap_or(u64::MAX);
    let mut stmt = cnx.prepare_cached(&query)?;
    let mut rows = stmt.query([from, to])?;
    let mut stuckthreads = Vec::new();
    let mut aggregate_buffer: HashMap<u64, (u64, u64, String, Option<String>, Option<u64>)> =
        HashMap::new();

    while let Some(row) = rows.next()? {
        let timestamp: u64 = row.get(0)?;
        let tid = row.get(1)?;
        let duration = row.get(2)?;
        let name = row.get(3)?;
        let request: Option<String> = row.get(4)?;
        let active = row.get(5)?;

        if let Some((begin_timestamp, _, begin_name, begin_request, begin_active)) =
            aggregate_buffer.get(&tid)
        {
            let end = if timestamp > *begin_timestamp + duration {
                timestamp
            } else {
                *begin_timestamp + duration
            };

            let request = begin_request.clone();
            stuckthreads.push(AggregatedStuckthread {
                tid,
                begin: Some(*begin_timestamp),
                end: Some(end),
                name: begin_name.to_string(),
                request,
                active_start: *begin_active,
                active_end: active,
                duration,
            });
            aggregate_buffer.remove(&tid);
        } else {
            if request.is_none() {
                stuckthreads.push(AggregatedStuckthread {
                    tid,
                    begin: None,
                    end: Some(timestamp),
                    name,
                    request: None,
                    active_start: None,
                    active_end: None,
                    duration,
                });
            } else {
                aggregate_buffer.insert(tid, (timestamp, duration, name, request, active));
            }
        }
    }

    for (tid, (begin_timestamp, begin_duration, begin_name, begin_request, begin_active)) in
        aggregate_buffer
    {
        stuckthreads.push(AggregatedStuckthread {
            tid,
            begin: Some(begin_timestamp),
            end: None,
            name: begin_name,
            duration: begin_duration,
            request: begin_request,
            active_start: begin_active,
            active_end: None,
        });
    }

    Ok(stuckthreads)
}

pub fn get_stuckthread_trace<'a>(
    cnx: &Connection,
    tid: u64,
    timestamp: u64,
) -> Result<Vec<Frame<'a>>, Error> {
    let query = format!(
        "SELECT {0}.method, {0}.source FROM {0} WHERE {0}.tid = $1 AND {0}.timestamp = $2 ORDER BY {0}.idx",
        Tables::StuckthreadTraces.into_str()
    );
    let mut stmt = cnx.prepare_cached(&query)?;
    let mut rows = stmt.query([tid, timestamp])?;
    let mut frames = Vec::new();
    while let Some(row) = rows.next()? {
        let frame = Frame {
            method: Cow::Owned(row.get(0)?),
            source: Cow::Owned(row.get(1)?),
        };
        frames.push(frame);
    }
    Ok(frames)
}
