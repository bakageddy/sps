use std::{collections::HashMap, fs, io, process::exit};

use clap::Parser;
use sps::stacktrace::StackTrace;
use sps::{database::Executor};
use sps::stuckthread::{StuckThread, StuckThreadMeta, StuckThreadMetaBegin, StuckThreadProducer};
use tracing::{Level, event};
fn main() -> io::Result<()> {
    tracing_subscriber::fmt().init();

    event!(Level::INFO, "Parsing Application Arguements");
    let args = sps::arg::AppArgs::parse();

    let abs = args.path.canonicalize()?;
    if !abs.is_dir() {
        exit(1);
    }

    let cnx;
    if args.db.is_none() {
        cnx = rusqlite::Connection::open_in_memory();
    } else {
        cnx = rusqlite::Connection::open(args.db.expect("Unreachable"));
    }

    if cnx.is_err() {
        println!("{cnx:#?}");
        exit(1);
    }

    let mut cnx = cnx.expect("Unreachable");
    let mut contents = String::new();
    let mut files = vec![];
    let schema = fs::read_to_string("schema.sql").unwrap();
    cnx.execute_batch(&schema).unwrap();

    for entry in abs.read_dir().unwrap() {
        let entry = entry.unwrap().path();
        let file_name = entry.file_name().unwrap().to_string_lossy();
        if file_name.starts_with("stuckthreads") {
            files.push(entry);
        }
    }

    files.sort_by_key(|p| {
        p.file_name()
            .and_then(|n| n.to_str())
            .and_then(|s| s.strip_prefix("stuckthreads"))
            .and_then(|s| s.strip_suffix(".txt"))
            .and_then(|n| n.parse::<u32>().ok())
    });

    files.reverse();
    for entry in files {
        contents.push_str(&fs::read_to_string(entry).unwrap());
    }

    event!(Level::INFO, "Parsing {abs:#?} directory");
    let _result = StuckThreadProducer::produce(&contents);
    event!(Level::INFO, "Finished Parsing {abs:#?} directory");

    if _result.is_none() {
        event!(Level::INFO, "Cannot parse the {abs:#?} directory");
    }

    let mut buffer: HashMap<u32, (StuckThreadMetaBegin, StackTrace)> = HashMap::new();
    let tx = cnx.transaction().unwrap();
    for entry in _result.expect("Unreachable") {
        match entry {
            Ok(entry) => {
                match entry.meta {
                    StuckThreadMeta::Begin(e) => {
                        buffer.insert(e.thread_id, (e, entry.st.expect("Unreachable")));
                    }
                    StuckThreadMeta::End(ref e) => {
                        if !buffer.contains_key(&e.thread_id) {
                            continue;
                        }

                        let (begin, st) = buffer.get(&e.thread_id).expect("Unreachable");

                        Executor::insert_stuckthread(&tx, begin, st, Some(e)).unwrap();
                    }
                };
            }
            Err(e) => {
                event!(Level::WARN, "Error during parsing {e:#?}");
            }
        }
    }
    tx.commit().unwrap();
    // println!("{result:#?}");

    Ok(())
}
