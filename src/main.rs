
use sps::{stacktrace::StackTrace, stuckthread::StuckThread};

fn main() {
    let stuck_thread = StuckThread::default();
    println!("{stuck_thread:#?}");
}
