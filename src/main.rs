use clap::Parser;
use sps::{
    arg::Command,
    persistence::store::Store,
    util::{self, LogFiles},
};
use tracing::{debug, info, warn};
// use std::net::Ipv4Addr;
// use std::process::exit;
// use std::str::FromStr;
// use tokio::net::TcpListener;

// use rmcp::{ServiceExt, transport};
// use sps::analysis::mcp::{AnalysisServer, init_db};
// use sps::database::Persistence;
// use sps::stuckthread::{Event, Stuc, StuckThread};
// use sps::threaddump::{ThreadDump, ThreadDumpStreamer};
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

    debug!("Parsing Application Arguements: {:?}", std::env::args());
    let args = sps::arg::AppArgs::parse();

    if let Command::Parse { path, database, .. } = args.command {
        if database.is_none() {
            warn!("No path provided for saving the result. Persisting in memory");
        }
        let pool = Store::open(database)?;
        let cnx = pool.get()?;
        let _ = Store::schema(&cnx)?;

        // TODO: Add a command line flag for the parser subcommand to hint at the files.

        info!("Starting memory mapping files from {path:?}");
        let LogFiles {
            cpumonitoring,
            cpumemstats,
            stuckqueries,
            stuckthreads,
            threaddumps,
            runningqueries,
        } = util::get_logfiles_sorted(util::get_entries(&path)?);

        // TODO: Add running queries parser and connection dump parser.
        std::thread::scope(|s| {
            s.spawn(|| {
                let _ =
                    util::parse_and_persist_stuckthreads(stuckthreads, pool.clone()).map_err(|e| {
                        warn!("Error during Parsing/Persisting Stuck Threads: {e}");
                    });
            });

            s.spawn(|| {
                let _ =
                    util::parse_and_persist_stuckqueries(stuckqueries, pool.clone()).map_err(|e| {
                        warn!("Error during Parsing/Persisting Stuck Queries: {e}");
                    });
            });

            s.spawn(|| {
                let _ = util::parse_and_persist_cpumonitoring(cpumonitoring, pool.clone()).map_err(
                    |e| {
                        warn!("Error during Parsing/Persisting CPUMonitoring: {e}");
                    },
                );
            });

            s.spawn(|| {
                let _ =
                    util::parse_and_persist_cpumemstats(cpumemstats, pool.clone()).map_err(|e| {
                        warn!("Error during Parsing/Persisting cpumemstats: {e}");
                    });
            });

            s.spawn(|| {
                let _ =
                    util::parse_and_persist_threaddump(threaddumps, pool.clone()).map_err(|e| {
                        warn!("Error during Parsing/Persisting thread dumps: {e}");
                    });
            });

            s.spawn(|| {
                let _ = util::parse_and_persist_running_queries(runningqueries, pool.clone())
                    .map_err(|e| {
                        warn!("Error during Parsing/Persisting Running Queries: {e}");
                    });
            });
        });
    } else {
        todo!("Not yet implemented");
    }
    Ok(())
}
