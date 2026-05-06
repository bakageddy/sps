use rmcp::transport::{
    StreamableHttpService,
    streamable_http_server::session::local::LocalSessionManager,
};
use std::collections::HashMap;
use std::process::exit;
use tokio::net::TcpListener;

use clap::Parser;
use rmcp::{ServiceExt, transport};
use sps::analysis::mcp::{AnalysisServer, init_db};
use sps::database::Persistence;
use sps::stuckthread::{StuckThread, StuckThreadMeta, StuckThreadStream};
use sps::threaddump::{ThreadDump, ThreadDumpStreamer};
use sps::util::{self, map_file};
use tracing::{debug, error, info, warn};

#[tokio::main]
async fn main() -> util::Result<()> {
    tracing_subscriber::fmt().init();

    info!("Parsing Application Arguements");
    let args = sps::arg::AppArgs::parse();

    // TODO: Bake in schema into the executable
    let mut cnx = Persistence::init_db(args.db)?;
    let tx = cnx.transaction()?;

    // WARN: Do not merge this and the following loop.
    // Lifetimes of Binary Stuckthread events can be tied to different maps

    let sorted_stuckthreads = util::get_sorted_stuckthreads(&args.path)?;
    let mut contents = vec![];
    for entry in &sorted_stuckthreads {
        let map = util::map_file(entry);
        let map = match map {
            Ok(map) => map,
            Err(e) => {
                warn!("Cannot map file {:?} due to {e}", &entry);
                continue;
            }
        };

        contents.push(map);
    }

    let mut buffer: HashMap<u32, StuckThread> = HashMap::new();
    let mut insert_count = 0;
    for (map, entry) in std::iter::zip(&contents, &sorted_stuckthreads) {
        info!("Parsing file {:?}", &entry);

        for chunk in StuckThreadStream(map) {
            // PARSE: Chunk
            let event = StuckThread::try_from(chunk);
            let event = match event {
                Ok(event) => event,
                Err(e) => {
                    warn!(
                        "Error during parsing chunk {:?} in {:?} : {e:?}",
                        chunk, entry
                    );
                    continue;
                }
            };

            // AGGREGATE: stuckthread bi-events
            match &event.meta {
                StuckThreadMeta::Begin(begin) => {
                    buffer.insert(begin.thread_id, event);
                }
                StuckThreadMeta::End(end) => {
                    if !buffer.contains_key(&end.thread_id) {
                        debug!(
                            "Cannot find start during aggregation, cannot find matching entry for event {end:?}"
                        );
                        continue;
                    }
                    let begin = buffer
                        .get(&end.thread_id)
                        .expect("SAFETY: checked in if statement above");
                    match Persistence::insert_stuckthread(&tx, begin, Some(&event)) {
                        Ok(_) => {}
                        Err(e) => warn!("Error during inserting stuckthread event: {e:?}"),
                    }
                    insert_count += 1;
                    buffer.remove(&end.thread_id);
                }
            };
        }
        info!("Finished Persisting stuckthread events for {:?}", &entry);
    }

    info!("Started Persisting leftover stuckthread events");
    for (_, event) in buffer {
        match Persistence::insert_stuckthread(&tx, &event, None) {
            Ok(_) => {}
            Err(e) => {
                warn!("Error during insert: {e:?}");
            }
        }
        insert_count += 1;
    }
    info!("Finished Persisting leftover stuckthread events");
    info!("Persisted {insert_count} stuckthread events");

    let threaddump_entries = util::get_sorted_threaddumps(&args.path)?;

    for entry in threaddump_entries {
        let map = map_file(&entry)?;
        info!("Parsing Thread Dumps from {entry:?}");
        for chunk in ThreadDumpStreamer(&map) {
            let chunk = chunk.trim();
            if chunk.is_empty() {
                continue;
            }

            let dump = match ThreadDump::try_from(chunk) {
                Ok(dump) => dump,
                Err(e) => {
                    warn!("Cannot parse {} due to {e:?}", &chunk[0..100]);
                    continue;
                }
            };

            match Persistence::insert_threaddump(&tx, &dump) {
                Ok(_) => {}
                Err(e) => warn!("ERROR during Persisting Threaddump: {e}"),
            };
        }
        info!("Finished Parsing and Persisting Thread Dumps from {entry:?}");
    }

    if let Err(e) = tx.commit() {
        warn!("Error during committing transaction: {e:?}");
    }

    init_db(cnx);
    if let Some(addr) = args.bind {
        let listener = match TcpListener::bind(&addr).await {
            Ok(l) => l,
            Err(e) => {
                error!("Cannot bind to {addr} due to {e:?}");
                exit(1);
            }
        };

        let mcp_service = StreamableHttpService::new(
            || Ok(AnalysisServer::new()),
            LocalSessionManager::default().into(),
            Default::default(),
        );

        // let service = StreamableHttpService::new(service_factory, session_manager, )
        let router = axum::Router::new().route_service("/mcp", mcp_service);
        info!("Starting MCP Server on {addr}");
        let _ = axum::serve(listener, router).await.unwrap();
        info!("Stopped MCP Server on {addr}");
    } else {
        info!("Starting MCP Server with stdio");
        let _ = AnalysisServer::new().serve(transport::stdio()).await;
        info!("Stopped MCP Server with stdout");
    }
    Ok(())
}
