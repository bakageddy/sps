use std::path::PathBuf;

// TODO: Web Server
// TODO: Online

#[derive(clap::Parser)]
pub struct AppArgs {
    #[arg(long, short)]
    pub path: PathBuf,
    #[arg(long("database"), short)]
    pub db: Option<PathBuf>,
    #[arg(long("bind"), short)]
    pub bind: Option<String>,
}
