use std::path::PathBuf;

// TODO: Web Server
// TODO: Online

#[derive(clap::Parser)]
pub struct AppArgs {
    pub path: PathBuf
}
