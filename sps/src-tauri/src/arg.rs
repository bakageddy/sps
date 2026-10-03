use std::path::PathBuf;

#[derive(clap::Parser)]
pub struct AppArgs {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(clap::Subcommand)]
pub enum Command {
    /// Parses the performance logs in `path` and persists it to --database
    Parse {
        /// Path to the logs
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Path to persist logs as a Database, Empty to persist it in memory
        #[arg(long, short)]
        database: Option<PathBuf>,
    },

    /// Launches the GUI with the --database
    Launch {
        /// Lauches the application with the given database path
        #[arg(long, short)]
        database: Option<PathBuf>,
    },

    /// Executes a `query` against --database and serializes result to `stdout` as json
    Query {
        /// SQL Query string to run against database
        #[arg(default_value = "")]
        sql: String,

        /// Path to the database after parsing logs
        #[arg(long, short)]
        database: PathBuf,

        /// Path to export the result as an csv file
        #[arg(long, short)]
        export: Option<PathBuf>,

        /// Limit the number of rows
        #[arg(long, short)]
        limit: Option<u64>,

        /// Skip --offset number of rows
        #[arg(long, short)]
        offset: Option<u64>
    },

    /// Prints the schema of the `database` to `stdout` in json
    Schema {
        /// Path to the database after parsing logs
        database: PathBuf,
    }
}
