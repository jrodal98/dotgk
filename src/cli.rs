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
        /// Optional path to cache file (defaults to config_dir/dotgk/cache.json)
        #[clap(long)]
        cache_path: Option<std::path::PathBuf>,
        /// Disable caching of the evaluation result
        #[clap(long)]
        no_cache: bool,
    },
    /// Set a value in the cache
    Set {
        name: String,
        value: bool,
        /// Optional path to cache file (defaults to config_dir/dotgk/cache.json)
        #[clap(long)]
        cache_path: Option<std::path::PathBuf>,
        /// Time-to-live in seconds for the cached value
        #[clap(long)]
        ttl: Option<u64>,
    },
    /// Sync all gatekeepers and cache results
    Sync {
        /// Optional path to cache file (defaults to config_dir/dotgk/cache.json)
        #[clap(long)]
        cache_path: Option<std::path::PathBuf>,
        /// Force re-evaluation of all gatekeepers, ignoring TTL
        #[clap(long)]
        force: bool,
    },
}
