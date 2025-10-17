// This is the main entry point for the Plant Care CLI application.
// 
// It demonstrates several Rust concepts:
// - Using the clap crate for command-line argument parsing
// - Async/await with tokio runtime
// - Database initialization and management
// - Dependency injection pattern
// - Error handling with anyhow::Result

use anyhow::Result;
use clap::Parser;
use plant_id::{
    service::DiagnosisService,
    controller,
    database::{DiagnosisRepository, PlantRepository},
    engine::KernelExecutor,
};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Main CLI structure - defines the available commands
/// 
/// The #[derive(Parser)] attribute from clap automatically generates
/// the command-line parsing code. This is an example of Rust's
/// powerful procedural macros.
#[derive(Parser)]
#[command(name = "plant-care")]
#[command(about = "AI-powered plant diagnosis CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Enum defining all available CLI commands
/// 
/// Each variant represents a different command the user can run.
/// The clap library will parse the command-line arguments and
/// match them to one of these variants.
#[derive(Parser)]
enum Commands {
    /// Add a new plant
    Add {
        /// Path to plant image
        #[arg(short, long)]
        image: String,
    },
    /// List all plants
    List,
    /// Diagnose a plant problem
    /// 
    /// This is the main diagnosis endpoint we implemented.
    /// Example: cargo run -- diagnose --plant-id <UUID> --prompt "leaves browning"
    Diagnose {
        /// Plant ID to diagnose
        #[arg(short, long)]
        plant_id: String,
        /// Description of the problem
        #[arg(short = 'P', long)]
        prompt: String,
    },
    /// Show diagnosis session details
    /// 
    /// Displays the conversation history and status of a diagnosis.
    /// Example: cargo run -- show-diagnosis --diagnosis-id <UUID>
    ShowDiagnosis {
        /// Diagnosis ID to display
        #[arg(short, long)]
        diagnosis_id: String,
    },
}

/// Main function - the entry point of the program
/// 
/// #[tokio::main] is a macro that:
/// - Sets up the tokio async runtime
/// - Allows us to use async/await in main()
/// - Handles the async event loop
#[tokio::main]
async fn main() -> Result<()> {
    // STEP 1: Parse command-line arguments
    // This uses the clap library to match user input to our Commands enum
    let cli = Cli::parse();

    // STEP 2: Initialize the database
    // Open (or create) the SQLite database file
    // This demonstrates Rust's Result type - ? propagates errors
    let conn = Connection::open("plant_id.db")?;

    // STEP 3: Create database tables if they don't exist
    // This is idempotent - safe to run multiple times
    
    // Create the plants table
    // Stores information about each plant in the user's collection
    conn.execute(
        "CREATE TABLE IF NOT EXISTS plants (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            care_instructions TEXT,
            watering_schedule TEXT,
            image_path TEXT,
            created_at TEXT NOT NULL
        )",
        [],
    )?;

    // Create the diagnoses table
    // Stores diagnosis sessions with conversation history
    // Note the FOREIGN KEY constraint linking to plants
    conn.execute(
        "CREATE TABLE IF NOT EXISTS diagnoses (
            id TEXT PRIMARY KEY,
            plant_id TEXT NOT NULL,
            status TEXT NOT NULL,
            context TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY(plant_id) REFERENCES plants(id)
        )",
        [],
    )?;

    // STEP 4: Wrap the connection in Arc<Mutex<T>> for thread-safe sharing
    // Why Arc<Mutex<T>>?
    // - Arc = Atomic Reference Counting: allows multiple owners
    // - Mutex = Mutual Exclusion: ensures only one thread accesses at a time
    // This is Rust's approach to safe concurrent access to shared data
    let conn = Arc::new(Mutex::new(conn));

    // STEP 5: Initialize repositories and services (Dependency Injection)
    // We create all the components and inject their dependencies
    
    // PlantRepository - handles database operations for plants
    // We clone the Arc to give it shared ownership of the connection
    let plant_repo = PlantRepository::new(Arc::clone(&conn));
    
    // DiagnosisRepository - handles database operations for diagnoses
    let diagnosis_repo = DiagnosisRepository::new(Arc::clone(&conn));
    
    // KernelExecutor - the AI brain that generates responses
    let kernel = KernelExecutor::new();
    
    // DiagnosisService - orchestrates the workflow
    // We move ownership of the repositories and kernel into the service
    let diagnosis_service = DiagnosisService::new(plant_repo, diagnosis_repo, kernel);

    // STEP 6: Match the parsed command and route to appropriate controller
    // Controllers handle the CLI endpoints and user interaction
    match cli.command {
        Commands::Add { image } => {
            // Route to plant controller's add endpoint
            controller::add_plant(image)?;
        }
        Commands::List => {
            // Route to plant controller's list endpoint
            controller::list_plants()?;
        }
        Commands::Diagnose { plant_id, prompt } => {
            // Parse the plant_id string into a Uuid
            // The ? operator returns early if parsing fails
            let plant_uuid = Uuid::parse_str(&plant_id)?;
            
            // Route to diagnosis controller's diagnose endpoint
            // We pass a reference (&) to the service because we don't need to move it
            // This is an async function, so we await it
            controller::diagnose(&diagnosis_service, plant_uuid, prompt).await?;
        }
        Commands::ShowDiagnosis { diagnosis_id } => {
            // Parse the diagnosis_id string into a Uuid
            let diag_uuid = Uuid::parse_str(&diagnosis_id)?;
            
            // Route to diagnosis controller's show_diagnosis endpoint
            // This is a synchronous function (no await needed)
            controller::show_diagnosis(&diagnosis_service, diag_uuid)?;
        }
    }

    // If we reach here, everything succeeded
    // The Ok(()) is returned to indicate success
    Ok(())
}