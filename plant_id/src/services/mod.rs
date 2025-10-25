/*!
 * SERVICES MODULE
 *
 * Application services that orchestrate business logic.
 * These implement use cases by coordinating between domain models,
 * repositories, and external adapters.
 */

// Declare service modules
pub mod diagnosis_service;
pub mod plant_service;

// Re-export service structs for easier access
pub use diagnosis_service::DiagnosisService;
pub use plant_service::PlantService;

