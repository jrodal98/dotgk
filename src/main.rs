mod cache;
mod cli;
mod evaluators;
mod gatekeeper;

use anyhow::Result;
use clap::Parser;
use cli::Args;
use cli::Command;
use gatekeeper::evaluate_gatekeeper_by_name;
use tracing::debug;
use tracing::info;
use tracing::instrument;

#[instrument]
fn evaluate_command(
    name: String,
    cache_path: Option<std::path::PathBuf>,
    no_cache: bool,
) -> Result<()> {
    info!("Evaluating gatekeeper: {}", name);

    let result = evaluate_gatekeeper_by_name(&name)?;
    info!("Evaluation result: {}", result);
    println!("{}", result);

    // Cache the result unless --no-cache is specified
    if !no_cache {
        if let Err(e) =
            cache::cache_evaluation_result(&name, result, cache_path, cache::UpdateType::Evaluate)
        {
            // Don't fail the command if caching fails, just log the error
            tracing::warn!("Failed to cache evaluation result: {}", e);
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    debug!("Parsed args: {:?}", args);

    match args.command {
        Command::Evaluate {
            name,
            cache_path,
            no_cache,
        } => evaluate_command(name, cache_path, no_cache),
        Command::Sync { cache_path } => cache::sync_command(cache_path),
    }
}
