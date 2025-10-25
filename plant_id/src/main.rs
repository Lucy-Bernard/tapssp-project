/*!
 * PLANT CARE CLI - Main Entry Point
 *
 * This is a CLI application for AI-driven plant identification, care scheduling,
 * and health diagnosis using external APIs (Plant.id and OpenRouter).
 */

// Module declarations - these tell Rust where to find our code modules
mod adapters;
mod cli;
mod config;
mod domain;
mod dto;
mod repositories;
mod services;

use anyhow::Result;
use clap::Parser;
use dotenv::dotenv;

use cli::Cli;
use config::Database;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    dotenv().ok();

    // Initialize logging
    env_logger::init();

    // Parse command-line arguments
    let cli = Cli::parse();

    // Initialize database connection
    let db = Database::new().await?;

    // Run database migrations to ensure tables exist
    db.migrate().await?;

    // Execute the CLI command
    cli.execute(db).await?;

    Ok(())
}
