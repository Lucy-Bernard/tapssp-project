/*!
 * CLI INTERFACE MODULE
 *
 * Defines the command-line interface structure using clap.
 * This is the primary adapter that receives user input and translates it
 * into service calls (following hexagonal architecture).
 */

mod commands;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::config::Database;

#[derive(Parser)]
#[command(
    name = "plant-care",
    version,
    about = "AI-driven plant care and diagnosis CLI",
    long_about = "Identify plants, generate care schedules, and diagnose plant health issues using AI"
)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new plant to your collection
    Add {
        /// Path to plant image file
        #[arg(short, long)]
        image: String,

        /// Optional plant name (if known)
        #[arg(short, long)]
        name: Option<String>,

        /// Latitude for location-based identification
        #[arg(long)]
        latitude: Option<f64>,

        /// Longitude for location-based identification
        #[arg(long)]
        longitude: Option<f64>,
    },

    /// List all plants in your collection
    List,

    /// Show details for a specific plant
    Show {
        /// Plant ID or name
        plant: String,
    },

    /// Delete a plant from your collection
    Delete {
        /// Plant ID or name
        plant: String,
    },

    /// Start an interactive diagnosis session for a plant
    Diagnose {
        /// Plant ID or name
        plant: String,

        /// Initial problem description
        #[arg(short, long)]
        problem: String,
    },

    /// View diagnosis history for a plant
    History {
        /// Plant ID or name
        plant: String,
    },

    /// Generate care schedule for a plant (without adding to collection)
    Care {
        /// Plant name
        name: String,
    },
}

impl Cli {
    pub async fn execute(self, db: Database) -> Result<()> {
        match self.command {
            Commands::Add {
                image,
                name,
                latitude,
                longitude,
            } => {
                commands::add_plant(db, image, name, latitude, longitude).await
            }
            Commands::List => commands::list_plants(db).await,
            Commands::Show { plant } => commands::show_plant(db, plant).await,
            Commands::Delete { plant } => commands::delete_plant(db, plant).await,
            Commands::Diagnose { plant, problem } => {
                commands::diagnose_plant(db, plant, problem).await
            }
            Commands::History { plant } => commands::show_history(db, plant).await,
            Commands::Care { name } => commands::generate_care(name).await,
        }
    }
}

