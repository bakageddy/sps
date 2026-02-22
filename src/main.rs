use std::{fs, io, process::exit};

use clap::Parser;
use sps::{stuckthread::{StuckThread, StuckThreadProducer}};
fn main() -> io::Result<()> {
    tracing_subscriber::fmt().init();

    let args = sps::arg::AppArgs::parse();
    

    let abs = args.path.canonicalize()?;
    if !abs.is_dir() {
        exit(1);
    }

    let mut contents = String::new();

    for entry in abs.read_dir().unwrap() {
        let entry = entry.unwrap().path();
        if !entry.file_name().unwrap().to_string_lossy().starts_with("stuckthreads") {
            continue;
        }
        contents.push_str(&fs::read_to_string(entry).unwrap());
    }

    let result = StuckThreadProducer::produce(&contents);
    println!("{result:#?}");

    
    Ok(())
}
