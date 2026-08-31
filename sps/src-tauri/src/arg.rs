use std::path::PathBuf;

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

    Launch {
        /// Lauches the application with the given database path
        #[arg(long, short)]
        database: Option<PathBuf>,
    },
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
