use std::path::PathBuf;

// TODO: Web Server
// TODO: Online

#[derive(clap::Parser)]
pub struct AppArgs {
    #[arg(long, short)]
    pub path: PathBuf,
    #[arg(long, short)]
    pub db: Option<PathBuf>,
}
