use std::{net::Ipv4Addr, path::PathBuf};

// TODO: Web Server
// TODO: Online

#[derive(clap::Parser)]
pub struct AppArgs {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(clap::Subcommand)]
pub enum Command {
    Parse {
        #[arg(long, short)]
        path: PathBuf,
        #[arg(long, short)]
        database: Option<PathBuf>,
    },
    MCP {
        #[arg(long("stdio"), short, action)]
        stdio: bool,
        #[arg(long, short, default_value = "127.0.0.1")]
        bind: Option<String>,
        #[arg(long, short, default_value = "8080")]
        port: Option<u16>,
        #[arg(long, short)]
        database: PathBuf,
    },
    // TODO: Add arguements when you finish the web api version
    Web,
}
