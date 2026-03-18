use std::collections::HashMap;

use clap::Parser;
use sps::stuckthread::{StuckThread, StuckThreadMeta, StuckThreadMetaBegin, StuckThreadStream};
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
    let mut buffer: HashMap<u32, (StuckThreadMetaBegin, StackTrace)> = HashMap::new();
    for (ref map, entry) in std::iter::zip(&contents, &sorted_stuckthreads) {
        info!("Parsing file {:?}", &entry);

        for chunk in StuckThreadStream(&map) {
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

            match event.meta {
                StuckThreadMeta::Begin(begin) => {
                    buffer.insert(
                        begin.thread_id,
                        (begin, event.st.expect("SAFETY: match in begin")),
                    );
                }
                StuckThreadMeta::End(ref end) => {
                    if !buffer.contains_key(&end.thread_id) {
                        warn!(
                            "Error during aggregation, cannot find matching entry for event {end:?}"
                        );
                        continue;
                    }
                    let (begin, st) = buffer
                        .get(&end.thread_id)
                        .expect("SAFETY: checked in if statement above");
                    match Persistence::insert_stuckthread(&tx, &begin, &st, Some(end)) {
                        Ok(_) => {}
                        Err(e) => warn!("Error during inserting stuckthread event: {e:?}"),
                    }
                    buffer.remove(&end.thread_id);
                }
            };
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
