
use std::fs;

use sps::{stacktrace::StackTrace, stuckthread::StuckThread};

fn main() {
    let stuckthread = fs::read_to_string("./sample/stuckthread.txt").unwrap();
    let stuck_thread = StuckThread::try_from(stuckthread.as_str());
    println!("{stuck_thread:#?}");
}
