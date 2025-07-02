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
    Evaluate { name: String },
    /// Sync all gatekeepers and cache results
    Sync {
        /// Optional path to cache file (defaults to config_dir/dotgk/cache.json)
        #[clap(long)]
        cache_path: Option<std::path::PathBuf>,
    },
}
