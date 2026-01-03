mod cache;
mod cli;
mod gatekeeper;
mod lua_executor;
mod settings;

use anyhow::Result;
use clap::Parser;
use cli::Args;
use cli::CacheAction;
use cli::Command;
use tracing::debug;
use tracing::info;
use tracing::instrument;
use tracing_subscriber::EnvFilter;

use crate::gatekeeper::load_and_evaluate_gatekeeper;

#[instrument]
fn evaluate_command(name: String, no_cache: bool) -> Result<()> {
    info!("Evaluating gatekeeper: {}", name);

    let gatekeeper_result = load_and_evaluate_gatekeeper(&name)?;
    let result = gatekeeper_result.value;
    info!("Evaluation result: {}", result);
    println!("{}", result);

    // Cache the result unless --no-cache is specified
    if !no_cache {
        let ttl = gatekeeper_result.ttl;

        if let Err(e) =
            cache::cache_result_with_ttl(&name, result, None, cache::UpdateType::Evaluate, ttl)
        {
            // Don't fail the command if caching fails, just log the error
            tracing::warn!("Failed to cache evaluation result: {}", e);
        }
    }

    Ok(())
}

#[instrument]
fn cache_command(action: CacheAction) -> Result<()> {
    match action {
        CacheAction::Enable { name } => {
            info!("Enabling cache format: {}", name);

            // Load current settings
            let mut settings = settings::load_settings().unwrap_or_else(|e| {
                debug!("Failed to load settings, using defaults: {}", e);
                settings::Settings::default()
            });

            // Check if format is already enabled
            if settings.enabled_cache_formats.contains(&name) {
                println!("Cache format '{}' is already enabled", name);
                return Ok(());
            }

            // Add the format to enabled list
            settings.enabled_cache_formats.push(name.clone());

            // Save updated settings
            settings::save_settings(&settings)?;

            println!("Enabled cache format '{}'", name);
            println!(
                "Current enabled formats: {}",
                settings.enabled_cache_formats.join(", ")
            );

            // Run sync to generate the newly enabled cache format
            info!("Running sync to generate newly enabled cache format");
            cache::sync_command(None, false)?;
        }
        CacheAction::Disable { name } => {
            info!("Disabling cache format: {}", name);

            // Load current settings
            let mut settings = settings::load_settings().unwrap_or_else(|e| {
                debug!("Failed to load settings, using defaults: {}", e);
                settings::Settings::default()
            });

            // Check if format is currently enabled
            if !settings.enabled_cache_formats.contains(&name) {
                println!("Cache format '{}' is not currently enabled", name);
                return Ok(());
            }

            // Remove the format from enabled list
            settings
                .enabled_cache_formats
                .retain(|format| format != &name);

            // Save updated settings
            settings::save_settings(&settings)?;

            println!("Disabled cache format '{}'", name);
            if settings.enabled_cache_formats.is_empty() {
                println!("No cache formats are currently enabled");
            } else {
                println!(
                    "Current enabled formats: {}",
                    settings.enabled_cache_formats.join(", ")
                );
            }
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
        Command::Cache { action } => cache_command(action),
    }
}
