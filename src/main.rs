use std::{collections::HashMap, fs, process::exit};

use clap::Parser;
use sps::stuckthread::{StuckThreadMeta, StuckThreadMetaBegin, StuckThreadProducer};
use sps::util;
use sps::{database::Executor, stacktrace::StackTrace};
use tracing::{Level, event};

fn main() -> util::Result<()> {
    tracing_subscriber::fmt().init();

    event!(Level::INFO, "Parsing Application Arguements");
    let args = sps::arg::AppArgs::parse();

    // TODO: Remove Unwrap
    // TODO: Bake in schema into the executable
    let mut cnx = Executor::init_db(args.db, "./schema.sql")?;

    let mut contents = String::new();
    let sorted_stuckthreads = util::get_sorted_stuckthreads(&args.path)?;
    for entry in sorted_stuckthreads {
        contents.push_str(&fs::read_to_string(entry)?);
    }

    event!(Level::INFO, "Parsing {:#?} directory", &args.path);
    let _result = StuckThreadProducer::produce(&contents);
    event!(Level::INFO, "Finished Parsing {:#?} directory", &args.path);

    if _result.is_none() {
        event!(Level::INFO, "Cannot parse the {:#?} directory", &args.path);
        exit(1);
    }

    let mut buffer: HashMap<u32, (StuckThreadMetaBegin, StackTrace)> = HashMap::new();
    let tx = cnx.transaction()?;
    for entry in _result.expect("Safety: checked") {
        match entry.meta {
            StuckThreadMeta::Begin(e) => {
                buffer.insert(e.thread_id, (e, entry.st.expect("SAFETY: match")));
            }
            StuckThreadMeta::End(ref e) => {
                if !buffer.contains_key(&e.thread_id) {
                    continue;
                }
                let (begin, st) = buffer.get(&e.thread_id).expect("SAFETY: checked");
                Executor::insert_stuckthread(&tx, begin, st, Some(e)).unwrap();
                _ = buffer.remove(&e.thread_id);
            }
        }
    }
    tx.commit()?;
    Ok(())
}
