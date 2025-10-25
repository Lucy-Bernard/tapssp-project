/*!
 * DOMAIN MODELS
 *
 * Core business entities that represent the application domain.
 * These are framework-agnostic and contain pure business logic.
 */

// Declare domain modules
pub mod care_schedule;
pub mod diagnosis_session;
pub mod plant;
pub mod enums;

// Re-export domain entities
pub use care_schedule::CareSchedule;
pub use diagnosis_session::DiagnosisSession;
pub use plant::Plant;

// Re-export enums for easier access
pub use enums::{DiagnosisStatus, DiagnosisAction};
