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
    /// Sync all gatekeepers and cache results
    Sync {
        /// Optional path to cache file (defaults to config_dir/dotgk/cache.json)
        #[clap(long)]
        cache_path: Option<std::path::PathBuf>,
    },
}
