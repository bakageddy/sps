use std::{borrow::Cow, collections::HashMap};

use duckdb::Connection;

use crate::{
    handlers::types::AggregatedStuckthreadMinimal, parser::stuckthread::Frame, store::{error::Error, tables::Tables},
};

pub fn get_stuckthreads_aggregate_minimal(
    cnx: &Connection,
) -> Result<Vec<AggregatedStuckthreadMinimal>, Error> {
    let query = format!(
        "SELECT timestamp, tid, duration, request FROM {} ORDER BY timestamp",
        Tables::Stuckthread.into_str()
    );
    let mut stmt = cnx.prepare_cached(&query)?;
    let mut rows = stmt.query([])?;
    let mut stuckthreads = Vec::new();
    let mut aggregate_buffer: HashMap<u64, (u64, u64)> = HashMap::new();
    while let Some(row) = rows.next()? {
        let timestamp = row.get(0)?;
        let tid = row.get(1)?;
        let duration = row.get(2)?;
        let request: Option<String> = row.get(3)?;
        if request.is_some() {
            let old = aggregate_buffer.insert(tid, (timestamp, duration));
            if let Some((prev_begin_timestamp, prev_begin_duration)) = old {
                stuckthreads.push(AggregatedStuckthreadMinimal {
                    timestamp: prev_begin_timestamp,
                    tid,
                    duration_ms: prev_begin_duration,
                });
            }
        } else {
            if let Some((begin_timestamp, _)) = aggregate_buffer.get(&tid) {
                stuckthreads.push(AggregatedStuckthreadMinimal {
                    timestamp: *begin_timestamp,
                    tid,
                    duration_ms: duration,
                });
                aggregate_buffer.remove(&tid);
            } else {
                stuckthreads.push(AggregatedStuckthreadMinimal {
                    timestamp: timestamp - duration,
                    tid,
                    duration_ms: duration,
                });
            }
        }
    }

    for (tid, (timestamp, duration)) in aggregate_buffer {
        stuckthreads.push(AggregatedStuckthreadMinimal {
            timestamp,
            tid,
            duration_ms: duration,
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
