use std::{collections::HashMap, fs};

use clap::Parser;
use sps::stuckthread::{StuckThreadMeta, StuckThreadMetaBegin, StuckThreadStream};
use sps::util;
use sps::{database::Persistence, stacktrace::StackTrace};
use tracing::{Level, event, info, warn};

fn main() -> util::Result<()> {
    tracing_subscriber::fmt().init();

    event!(Level::INFO, "Parsing Application Arguements");
    let args = sps::arg::AppArgs::parse();

    // TODO: Bake in schema into the executable
    let mut cnx = Persistence::init_db(args.db, "./schema.sql")?;

    let sorted_stuckthreads = util::get_sorted_stuckthreads(&args.path)?;
    let mut contents = vec![];
    let mut buffer: HashMap<u32, (StuckThreadMetaBegin, StackTrace)> = HashMap::new();
    for entry in &sorted_stuckthreads {
        let map = util::map_file(&entry);
        let map = match map {
            Ok(map) => map,
            Err(e) => {
                warn!("Cannot map file {:?} due to {e}", &entry);
                continue;
            }
        };

        contents.push(map);
    }

    let tx = cnx.transaction()?;
    for (ref map, entry) in std::iter::zip(&contents, &sorted_stuckthreads) {
        info!("Parsing file {:?}", &entry);

        let events = match StuckThreadStream::parse(&map) {
            Ok(events) => events,
            Err(e) => {
                eprintln!("Error during parsing {:?} : {e:?}", &entry);
                continue;
            }
        };

        info!("Finished parsing file {:?}", &entry);

        info!("Started Persisting stuckthread events for {:?}", &entry);
        for event in events {
            let event = match event {
                Ok(event) => event,
                Err(e) => {
                    warn!("Error during parsing {:?}: {e}", &entry);
                    continue;
                }
            };

            match event.meta {
                StuckThreadMeta::Begin(e) => {
                    buffer.insert(e.thread_id, (e, event.st.expect("SAFETY: match")));
                }

                StuckThreadMeta::End(ref e) => {
                    if !buffer.contains_key(&e.thread_id) {
                        continue;
                    }
                    let (begin, st) = buffer.get(&e.thread_id).expect("SAFETY: checked");

                    match Persistence::insert_stuckthread(&tx, begin, st, Some(e)) {
                        Ok(_) => {}
                        Err(e) => warn!("Error during insert: {e:?}"),
                    }
                    _ = buffer.remove(&e.thread_id);
                }
            }
        }
        info!("Finished Persisting stuckthread events for {:?}", &entry);
    }

    info!("Started Persisting leftover stuckthread events");
    for (_, (begin, st)) in buffer {
        match Persistence::insert_stuckthread(&tx, &begin, &st, None) {
            Ok(_) => {}
            Err(e) => {
                warn!("Error during insert: {e:?}");
            }
        }
    }
    info!("Finished Persisting leftover stuckthread events");

    let _ = tx.commit()?;
    Ok(())
}
