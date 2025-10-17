// Controller module - handles CLI endpoints
//
// Controllers are responsible for:
// - Receiving user input from the CLI
// - Calling appropriate services to handle business logic
// - Formatting and displaying output to the user
//
// This follows the MVC (Model-View-Controller) pattern where:
// - Models: data structures (models/)
// - Views: CLI output (println! statements in controllers)
// - Controllers: request handlers (this module)

pub mod diagnosis_controller;
pub mod plant_controller;

// Re-export controller functions for easy access
pub use diagnosis_controller::{diagnose, show_diagnosis};
pub use plant_controller::{add_plant, list_plants};

