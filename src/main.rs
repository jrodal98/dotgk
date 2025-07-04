mod cache;
mod cli;
mod evaluators;
mod gatekeeper;

use anyhow::Result;
use clap::Parser;
use cli::Args;
use cli::Command;
use tracing::debug;
use tracing::info;
use tracing::instrument;
use tracing_subscriber::EnvFilter;

use crate::gatekeeper::Gatekeeper;

#[instrument]
fn evaluate_command(name: String, no_cache: bool) -> Result<()> {
    info!("Evaluating gatekeeper: {}", name);

    let gatekeeper = Gatekeeper::from_name(&name)?;
    let result = gatekeeper.evaluate()?;
    info!("Evaluation result: {}", result);
    println!("{}", result);

    // Cache the result unless --no-cache is specified
    if !no_cache {
        let ttl = gatekeeper.ttl;

        if let Err(e) =
            cache::cache_result_with_ttl(&name, result, None, cache::UpdateType::Evaluate, ttl)
        {
            // Don't fail the command if caching fails, just log the error
            tracing::warn!("Failed to cache evaluation result: {}", e);
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    // Set different default log levels for debug vs release builds
    let default_level = if cfg!(debug_assertions) {
        "info" // Debug builds default to info level
    } else {
        "error" // Release builds default to error level
    };

    // Initialize tracing with the appropriate default level
    // RUST_LOG environment variable can still override this default
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_level));

    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    let args = Args::parse();
    debug!("Parsed args: {:?}", args);

    match args.command {
        Command::Evaluate { name, no_cache } => evaluate_command(name, no_cache),
        Command::Get { name } => cache::get_command(name, None),
        Command::Set { name, value, ttl } => {
            let parsed_value = match value.to_lowercase().as_str() {
                "true" | "1" | "yes" | "on" => true,
                "false" | "0" | "no" | "off" => false,
                _ => {
                    eprintln!(
                        "Invalid boolean value '{}'. Use: true, false, 1, 0, yes, no, on, or off",
                        value
                    );
                    std::process::exit(1);
                }
            };
            cache::set_command(name, parsed_value, None, ttl)
        }
        Command::Sync { force } => cache::sync_command(None, force),
        Command::Rm { name, file } => cache::rm_command(name, None, file),
    }
}
