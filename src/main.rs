mod cli;
mod evaluators;
mod gatekeeper;
use clap::Parser;
use anyhow::{Context, Result};
use cli::{Args, Command};
use gatekeeper::{evaluate_gatekeeper, get_gatekeeper_path, Gatekeeper};
use tracing::{debug, error, info, instrument};

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();
    match args.command {
        Command::Evaluate { name } => evaluate_command(name),
    }
}

#[instrument]
fn evaluate_command(name: String) -> Result<()> {
    info!("Evaluating gatekeeper: {}", name);
    let gatekeeper_path = get_gatekeeper_path(&name)
        .with_context(|| format!("Failed to get gatekeeper path for '{}'", name))?;
    debug!("Gatekeeper path: {:?}", gatekeeper_path);

    if !gatekeeper_path.exists() {
        error!("Gatekeeper '{}' not found at {:?}", name, gatekeeper_path);
        anyhow::bail!("Gatekeeper '{}' not found at {:?}", name, gatekeeper_path);
    }

    let gatekeeper_content = std::fs::read_to_string(&gatekeeper_path)
        .with_context(|| format!("Failed to read gatekeeper '{}'", name))?;
    debug!("Gatekeeper content read successfully");

    let gatekeeper: Gatekeeper = serde_json::from_str(&gatekeeper_content)
        .with_context(|| format!("Failed to parse gatekeeper '{}'", name))?;
    debug!("Gatekeeper parsed successfully");

    let result = evaluate_gatekeeper(&gatekeeper);
    info!("Evaluation result: {}", result);
    println!("{}", result);
    Ok(())
}
