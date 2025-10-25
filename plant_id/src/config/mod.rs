/*!
 * CONFIGURATION MODULE
 *
 * Handles application configuration including database setup,
 * environment variables, and other infrastructure concerns.
 */

// Declare config modules
pub mod database;

// Re-export main configuration types
pub use database::Database;

// Re-export utility functions for environment variables
pub use database::get_env;
