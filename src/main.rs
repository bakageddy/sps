use sps::ingest::stuckquery_pgsql::StuckQueryTableIteratorPGSQL;
use sps::parser::stuckquery_pgsql::StuckQueryTable;
use sps::parser::threaddump::ThreadDump;
// use rmcp::transport::{
//     StreamableHttpService, streamable_http_server::session::local::LocalSessionManager,
// };
use sps::parser::stacktrace::Trace;
use sps::persistence::store::Store;
use sps::{arg::Command, ingest::threaddump::ThreadDumpIterator};
use std::collections::HashMap;
// use std::net::Ipv4Addr;
// use std::process::exit;
// use std::str::FromStr;
// use tokio::net::TcpListener;

use clap::Parser;
// use rmcp::{ServiceExt, transport};
// use sps::analysis::mcp::{AnalysisServer, init_db};
// use sps::database::Persistence;
use sps::ingest::stuckthread::StuckThreadIterator;
use sps::parser::stuckthread::*;
// use sps::stuckthread::{Event, Stuc, StuckThread};
// use sps::threaddump::{ThreadDump, ThreadDumpStreamer};
use sps::util;
use tracing::{debug, info, warn};
// // #[tokio::main]
// fn main() -> util::Result<()> {
//     tracing_subscriber::fmt().init();
//
//     info!("Parsing Application Arguements: {:?}", std::env::args());
//     let args = sps::arg::AppArgs::parse();
//
//     if let Command::Parse { path, database } = args.command {
//         if database.is_none() {
//             warn!("No path provided for saving the result. Persisting in memory");
//         }
//
//         // let mut cnx = Persistence::init_db(database.as_ref())?;
//         // let tx = cnx.transaction()?;
//
//         let mut cnx = Store::open(database.as_ref())?;
//         let _ = Store::schema(&cnx)?;
//         let tx = cnx.transaction()?;
//         let mut stacktrace_elements_appender = tx.appender("stacktrace_elements")?;
//         let mut stuckthread_appender = tx.appender("stuckthread")?;
//
//         info!("Starting memory mapping files from {path:?}");
//         let sorted_stuckthreads = util::get_sorted_stuckthreads(&path)?;
//         let maps = sorted_stuckthreads
//             .iter()
//             .filter_map(|e| {
//                 util::map_file(e)
//                     .inspect_err(|er| warn!("cannot map file {:?} due to {er}", e))
//                     .ok()
//             })
//             .collect::<Vec<_>>();
//         info!("Finished mapping files from {path:?}");
//
//         let events = std::iter::zip(&sorted_stuckthreads, &maps)
//             .map(|(entry, m)| {
//                 info!("Parsing stuckthread log: {entry:?}");
//                 StuckThreadStream(m)
//             })
//             .flatten()
//             .map(StuckThread::try_from)
//             .filter_map(|ev| {
//                 ev.inspect_err(|er| warn!("Error during parsing chunk {er:?}"))
//                     .ok()
//             })
//             .collect::<Vec<_>>();
//         info!("Finished Parsing {} stuckthread events", events.len());
//
//         let threaddump_entries = util::get_sorted_threaddumps(&path)?;
//         let maps = threaddump_entries
//             .iter()
//             .filter_map(|ent| {
//                 util::map_file(&ent)
//                     .inspect_err(|err| warn!("Failed to map file {ent:?} due to {err:?}"))
//                     .ok()
//             })
//             .collect::<Vec<_>>();
//
//         let dumps = std::iter::zip(&threaddump_entries, &maps)
//             .map(|(entry, map)| {
//                 info!("Parsing threaddump log file: {entry:?}");
//                 ThreadDumpStreamer(&map)
//             })
//             .flatten()
//             .map(|c| c.trim())
//             .filter(|c| !c.is_empty())
//             .filter_map(|c| {
//                 ThreadDump::try_from(c)
//                     .inspect_err(|e| warn!("Cannot parse \"{}\" due to {e:?}", c))
//                     .ok()
//             })
//             .collect::<Vec<_>>();
//
//         let mut aggregate_count = 0;
//         let mut aggregator: HashMap<u32, StuckThread> = HashMap::new();
//         debug!("Started Aggregating Stuckthread events");
//         for event in events {
//             match &event.event {
//                 Event::Begin(begin, _) => {
//                     aggregator.insert(begin.thread_id, event);
//                 }
//                 Event::End(end) => {
//                     if let Some(begin) = aggregator.get(&end.thread_id) {
//                         let Event::Begin(ref begin, ref st) = begin.event else {
//                             panic!("Unreachable")
//                         };
//                         let _ = Store::insert_stuckthread(
//                             &mut stuckthread_appender,
//                             &tx,
//                             &mut stacktrace_elements_appender,
//                             &begin,
//                             &st,
//                             Some(&end),
//                         )
//                         .inspect_err(|e| warn!("Error during inserting stuckthread event: {e:?}"));
//                         // let _ = Persistence::insert_stuckthread(&tx, begin, Some(&event))
//                         //     .inspect_err(|e| {
//                         //         warn!("Error during inserting stuckthread event: {e:?}")
//                         //     });
//                         aggregate_count += 1;
//                         aggregator.remove(&end.thread_id);
//                     } else {
//                         debug!(
//                             "Cannot find start during aggregation, cannot find matching entry for event {end:?}"
//                         );
//                     }
//                 }
//             }
//         }
//
//         for (_, event) in aggregator {
//             let Event::Begin(begin, st) = event.event else {
//                 panic!("Unreachable")
//             };
//             let _ = Store::insert_stuckthread(&mut stuckthread_appender, &tx, &mut stacktrace_elements_appender, &begin, &st, None)
//                 .inspect_err(|e| warn!("Error during inserting stuckthread event: {e:?}"));
//         }
//         stacktrace_elements_appender.flush()?;
//         stuckthread_appender.flush()?;
//         drop(stuckthread_appender);
//         drop(stacktrace_elements_appender);
//
//         info!("Finished Persisting {aggregate_count} stuckthread events");
//
//         // for dump in dumps {
//         //     let _ = Persistence::insert_threaddump(&tx, &dump)
//         //         .inspect_err(|e| warn!("Error during persisting thread dump: {e:?}"));
//         // }
//
//         if let Err(e) = tx.commit() {
//             warn!("Error during committing transaction: {e:?}");
//         }
//
//         info!(
//             "Persisted Stuckthread events and Thread dumps from {path:?} to {:?}",
//             database.unwrap_or("Memory".to_owned().into())
//         )
//     } else {
//         todo!()
//     }
//     // } else if let Command::MCP {
//     //     stdio,
//     //     bind,
//     //     database,
//     //     port,
//     // } = args.command
//     // {
//     //     let cnx = Persistence::init_db(database.into())?;
//     //     init_db(cnx);
//     //     if stdio {
//     //         info!("Starting MCP Server with stdio");
//     //         AnalysisServer::new()
//     //             .serve(transport::stdio())
//     //             .await
//     //             .unwrap();
//     //         info!("Stopped MCP Server with stdout");
//     //     } else if let Some(raw_addr) = bind {
//     //         let addr = Ipv4Addr::from_str(&raw_addr);
//     //         if let Err(e) = addr {
//     //             error!(
//     //                 "FAILED to parse {raw_addr} due to {}. Defaulting to localhost",
//     //                 e.to_string()
//     //             );
//     //             exit(1);
//     //         }
//     //         let addr = addr.unwrap_or(Ipv4Addr::LOCALHOST);
//     //         let port = port.unwrap_or(8080);
//     //
//     //         let listener = match TcpListener::bind((addr, port)).await {
//     //             Ok(l) => l,
//     //             Err(e) => {
//     //                 error!("Cannot bind to {addr}:{port} due to {e:?}");
//     //                 exit(1);
//     //             }
//     //         };
//     //
//     //         let mcp_service = StreamableHttpService::new(
//     //             || Ok(AnalysisServer::new()),
//     //             LocalSessionManager::default().into(),
//     //             Default::default(),
//     //         );
//     //
//     //         // let service = StreamableHttpService::new(service_factory, session_manager, )
//     //         let router = axum::Router::new().route_service("/mcp", mcp_service);
//     //         info!("Starting MCP Server on {addr}:{port}");
//     //         axum::serve(listener, router).await.unwrap();
//     //         info!("Stopped MCP Server on {addr}:{port}");
//     //     } else {
//     //         error!(
//     //             "--stdio must be provided or --bind should be provided to start the mcp server in stdio or http sse server. Exiting"
//     //         );
//     //         exit(1);
//     //     }
//     // } else if let Command::Web = args.command {
//     //     error!(
//     //         "Web Server not yet implemented in this version. Update to the latest version (if any)"
//     //     );
//     // }
//     Ok(())
// }

fn main() -> util::Result<()> {
    tracing_subscriber::fmt().init();

    info!("Parsing Application Arguements: {:?}", std::env::args());
    let args = sps::arg::AppArgs::parse();

    if let Command::Parse { path, database } = args.command {
        if database.is_none() {
            warn!("No path provided for saving the result. Persisting in memory");
        }
        let cnx = Store::open(database.as_ref())?;
        let _ = Store::schema(&cnx)?;

        let mut stuckthread = cnx.appender("stuckthread")?;
        let mut stacktrace = cnx.appender("stacktrace")?;
        let mut threads = cnx.appender("thread")?;
        let mut stuckquery_pgsql = cnx.appender("stuckquery_pgsql")?;

        info!("Starting memory mapping files from {path:?}");
        // let sorted_stuckthreads = util::get_sorted_stuckthreads(&path)?;
        let sorted_threaddumps = util::get_sorted_threaddumps(&path)?;
        let sorted_stuckquery = util::get_sorted_stuckqueries(&path)?;
        // let mut stuckthread_maps = Vec::with_capacity(10);
        // let mut threaddump_maps = Vec::with_capacity(10);
        let mut stuckquery_maps = Vec::with_capacity(5);
        // let mut events = Vec::with_capacity(10000);
        // let mut dumps = Vec::with_capacity(20);
        let mut stuckquery_pgsql_tables = Vec::with_capacity(100);

        // for entry in &sorted_stuckthreads {
        //     let map = util::map_file(entry)
        //         .inspect_err(|er| warn!("cannot map file {:?} due to {er}", entry))
        //         .unwrap();
        //     stuckthread_maps.push(map);
        // }
        //
        // for entry in &sorted_threaddumps {
        //     let map = util::map_file(entry)
        //         .inspect_err(|er| warn!("cannot map file {:?} due to {er}", entry))
        //         .unwrap();
        //     threaddump_maps.push(map);
        // }

        for entry in &sorted_stuckquery {
            let map = util::map_file(entry)
                .inspect_err(|er| warn!("Cannot map file {:?} due to {er}", entry))
                .unwrap();
            stuckquery_maps.push(map);
        }

        info!("Finished mapping files from {path:?}");

        // for (entry, map) in std::iter::zip(&sorted_stuckthreads, &stuckthread_maps) {
        //     info!("Parsing: {entry:?}");
        //     for chunk in StuckThreadIterator(map) {
        //         let stuckthread = StuckThread::try_from(chunk)
        //             .inspect_err(|er| warn!("Error during parsing chunk {er:?}"))
        //             .unwrap();
        //         events.push(stuckthread);
        //     }
        // }
        //
        // info!("Finished Parsing {} stuckthread events", events.len());
        //
        // for (entry, map) in std::iter::zip(&sorted_threaddumps, &threaddump_maps) {
        //     info!("Parsing: {entry:?}");
        //     for chunk in ThreadDumpIterator(map) {
        //         if chunk.is_empty() {
        //             continue;
        //         }
        //         let threaddump = match ThreadDump::try_from(chunk) {
        //             Ok(o) => o,
        //             Err(e) => {
        //                 warn!("Error during parsing chunk {e:?}");
        //                 continue;
        //             }
        //         };
        //         dumps.push(threaddump)
        //     }
        // }
        //
        // info!("Finished Parsing {} threaddumps", dumps.len());

        for (entry, map) in std::iter::zip(&sorted_stuckquery, &stuckquery_maps) {
            info!("Parsing: {entry:?}");
            for chunk in StuckQueryTableIteratorPGSQL(map) {
                if chunk.is_empty() {
                    continue;
                }

                let table = match StuckQueryTable::try_from(chunk) {
                    Ok(table) => table,
                    Err(e) => {
                        warn!("Error during parsing chunk: {e:?}");
                        continue;
                    }
                };
                stuckquery_pgsql_tables.push(table);
            }
        }
        info!("Finished Parsing {} PGSQL stuckquery tables", stuckquery_pgsql_tables.len());

        // let mut aggregator: HashMap<u64, &StuckThread> = HashMap::new();
        // let mut aggregates: Vec<(&Begin, &Trace, Option<&End>)> = Vec::with_capacity(100);
        // debug!("Started Aggregating Stuckthread events");
        //
        // for event in &events {
        //     match &event.0 {
        //         Event::Begin(begin, _) => {
        //             aggregator.insert(begin.tid, event);
        //         }
        //         Event::End(end) => {
        //             if let Some(StuckThread(Event::Begin(begin, st))) = aggregator.get(&end.tid) {
        //                 aggregates.push((begin, st, Some(end)))
        //             }
        //         }
        //     }
        // }

        // for (_, event) in aggregator {
        //     if let StuckThread(Event::Begin(begin, st)) = event {
        //         aggregates.push((begin, st, None))
        //     }
        // }

        // info!("Started persisting {} aggregated events", aggregates.len());
        // for (begin, st, end) in &aggregates {
        //     let _ = Store::insert_stuckthread(&mut stuckthread, &mut stacktrace, begin, st, *end)?;
        // }
        // info!("Finished persisting {} aggregated events", aggregates.len());

        // info!("Started persisting {} threaddumps", dumps.len());
        // for dump in &dumps {
        //     let _ = Store::insert_threaddump(&mut threads, &mut stacktrace, dump)?;
        // }
        // info!("Finished persisting {} threaddumps", dumps.len());

        info!("Started persisting {} PGSQL stuckquery tables", stuckquery_pgsql_tables.len());
        for table in &stuckquery_pgsql_tables {
            let _ = Store::insert_stuckquery_pgsql_table(&mut stuckquery_pgsql, &table)?;
        }
        info!("Finished persisting {} PGSQL stuckquery tables", stuckquery_pgsql_tables.len());

        stuckthread.flush()?;
        stacktrace.flush()?;
        threads.flush()?;
        stuckquery_pgsql.flush()?;

        drop(stuckthread);
        drop(stacktrace);
        drop(threads);
        drop(stuckquery_pgsql);
        _ = cnx.close();
    } else {
        todo!("Not yet implemented");
    }
    Ok(())
}
