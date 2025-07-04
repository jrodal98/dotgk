use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(clap::Subcommand, Debug)]
pub enum Command {
    /// Evaluate a gatekeeper
    Evaluate {
        name: String,
        /// Disable caching of the evaluation result
        #[clap(long)]
        no_cache: bool,
    },
    /// Get a gatekeeper value from cache, evaluating if expired or missing
    Get {
        /// Gatekeeper name (if not provided, shows all gatekeepers)
        name: Option<String>,
    },
    /// Set a value in the cache
    Set {
        name: String,
        /// Value to set (true or false)
        value: String,
        /// Time-to-live in seconds for the cached value
        #[clap(long)]
        ttl: Option<u64>,
    },
    /// Sync all gatekeepers and cache results
    Sync {
        /// Force re-evaluation of all gatekeepers, ignoring TTL
        #[clap(long)]
        force: bool,
    },
    /// Remove a gatekeeper entry and optionally its file
    Rm {
        name: String,
        /// Also remove the gatekeeper JSON file if it exists
        #[clap(long)]
        file: bool,
    },
}
