/*!
 * REPOSITORIES MODULE
 *
 * Data access layer that handles persistence operations.
 * These are secondary adapters in hexagonal architecture,
 * abstracting database operations from the business logic.
 */

// Declare repository modules
pub mod diagnosis_repository;
pub mod plant_repository;

// Re-export repository structs for easier access
pub use diagnosis_repository::DiagnosisRepository;
pub use plant_repository::PlantRepository;

