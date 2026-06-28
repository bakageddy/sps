use std::path::PathBuf;

// TODO: Web Server

#[derive(clap::Parser)]
pub struct AppArgs {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(clap::Subcommand)]
pub enum Command {
    Parse {
        /// Path to the logs
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Path to persist logs as a Database
        #[arg(long, short)]
        database: Option<PathBuf>,

        /// Hints the parser of Database Log Type (runningqueries, stuckqueries)
        #[arg(long, short('t'), value_enum)]
        db_kind: Option<DBKind>,

        /// Hints the parser of OS Log Type (cpumemstats)
        #[arg(long, short('o'), value_enum)]
        os_kind: Option<OSKind>,
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

#[derive(clap::ValueEnum, Clone)]
pub enum DBKind {
    PGSQL,
    MSSQL,
}

#[derive(clap::ValueEnum, Clone)]
pub enum OSKind {
    Windows,
    UNIX,
}
