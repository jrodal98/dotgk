mod cli;
mod evaluators;
mod gatekeeper;
use clap::Parser;

use anyhow::{Context, Result};
use cli::{Args, Command};
use gatekeeper::{evaluate_gatekeeper, get_gatekeeper_path, Gatekeeper};

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Evaluate { name } => {
            let gatekeeper_path = get_gatekeeper_path(&name)?;
            if !gatekeeper_path.exists() {
                anyhow::bail!("Gatekeeper '{}' not found at {:?}", name, gatekeeper_path);
            }
            let gatekeeper_content = std::fs::read_to_string(&gatekeeper_path)
                .with_context(|| format!("Failed to read gatekeeper '{}'", name))?;
            let gatekeeper: Gatekeeper = serde_json::from_str(&gatekeeper_content)
                .with_context(|| format!("Failed to parse gatekeeper '{}'", name))?;
            let result = evaluate_gatekeeper(&gatekeeper);
            println!("{}", result);
            Ok(())
        }
    }
}
