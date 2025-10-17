// Repository module - contains all data access logic
//
// This module organizes database operations using the Repository Pattern.
// Each entity (Plant, Diagnosis, etc.) has its own repository for database operations.

mod diagnosis_repository;
mod plant_repository;

// Re-export repositories for easy access
pub use diagnosis_repository::DiagnosisRepository;
pub use plant_repository::PlantRepository;

