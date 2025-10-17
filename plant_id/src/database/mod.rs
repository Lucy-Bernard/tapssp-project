// Database module - handles all data persistence operations
//
// This module is organized into:
// - repository/ - Contains repository implementations for each entity
// - Other database-related utilities (migrations, connections, etc.)

pub mod repository;
mod chats;

// Re-export repositories for convenient access
pub use repository::{DiagnosisRepository, PlantRepository};
