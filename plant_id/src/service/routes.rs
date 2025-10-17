// This module contains the DiagnosisService, which is the "service layer" of the application.
// The service layer sits between the CLI commands and the database/business logic.
// It orchestrates the workflow by coordinating between repositories and the kernel executor.

use anyhow::{anyhow, Result};
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    database::{DiagnosisRepository, PlantRepository},
    engine::KernelExecutor,
    models::{DiagnosisContext, DiagnosisSession, DiagnosisStatus},
};

/// Service layer for diagnosis operations
///
/// This struct demonstrates the "Dependency Injection" pattern - it holds references
/// to the repositories and kernel it needs, making it testable and modular.
///
/// Why this design?
/// - Separates business logic from data access (repositories)
/// - Makes testing easier (you can mock the repositories)
/// - Follows the Single Responsibility Principle
pub struct DiagnosisService {
    plant_repo: PlantRepository,        // For accessing plant data
    diagnosis_repo: DiagnosisRepository, // For accessing diagnosis data
    kernel: KernelExecutor,              // For running the AI diagnosis logic
}

impl DiagnosisService {
    /// Constructor that takes ownership of the dependencies
    ///
    /// This follows Rust's ownership model - we move the repositories and kernel
    /// into the service, and the service becomes responsible for them.
    pub fn new(
        plant_repo: PlantRepository,
        diagnosis_repo: DiagnosisRepository,
        kernel: KernelExecutor,
    ) -> Self {
        Self {
            plant_repo,
            diagnosis_repo,
            kernel,
        }
    }

    /// Create a new diagnosis session for a plant
    ///
    /// This is the main function that implements the diagnosis endpoint logic.
    /// It follows a clear workflow with numbered steps for clarity.
    ///
    /// # Arguments
    /// * `&self` - Borrows the service (doesn't take ownership)
    /// * `plant_id` - UUID of the plant to diagnose
    /// * `prompt` - User's problem description
    ///
    /// # Returns
    /// * `Ok((diagnosis_id, ai_question, status))` - Tuple with the results
    /// * `Err` - If plant not found or any operation fails
    pub async fn create_diagnosis(
        &self,
        plant_id: Uuid,
        prompt: String,
    ) -> Result<(Uuid, String, DiagnosisStatus)> {
        // STEP 1: Validate that the plant exists
        // We query the database using the plant repository
        // The `?` operator propagates errors up if the database query fails
        let plant = self
            .plant_repo
            .get_by_id(plant_id)
            // map_err converts the database error into an anyhow error with context
            .map_err(|e| anyhow!("Database error: {}", e))?;

        // Check if the plant was found (Option is None if not found)
        // This is our "404 Not Found" equivalent error
        if plant.is_none() {
            return Err(anyhow!("Plant with ID {} not found", plant_id));
        }

        // STEP 2: Create a new diagnosis session object
        // Generate a unique UUID for this diagnosis session
        let diagnosis_id = Uuid::new_v4();
        // Get the current timestamp for created_at and updated_at
        let now = Utc::now();

        // Build the DiagnosisSession struct
        // We make it mutable because the kernel will modify it
        let mut session = DiagnosisSession {
            id: diagnosis_id,
            plant_id,
            status: DiagnosisStatus::Processing, // Initial status
            context: DiagnosisContext {
                initial_prompt: prompt.clone(), // Store the user's original problem
                conversation_history: Vec::new(), // Empty at first, kernel will add messages
                state: HashMap::new(), // For storing additional state data
            },
            created_at: now,
            updated_at: now,
        };

        // STEP 3: Run the first cycle of the kernel loop
        // This is where the AI magic happens - the kernel:
        // - Adds the user's message to conversation history
        // - Generates a follow-up question based on the prompt
        // - Adds the AI's response to conversation history
        // - Updates the session status
        // We pass a mutable reference (&mut) so the kernel can modify the session
        let ai_question = self.kernel.run_initial_cycle(&mut session).await?;

        // STEP 4: Persist the diagnosis session to the database
        // Now that we have the complete session (with conversation history),
        // save it to the database so we can retrieve it later
        self.diagnosis_repo
            .create(&session)
            .map_err(|e| anyhow!("Failed to save diagnosis: {}", e))?;

        // STEP 5: Return the results as a tuple
        // The CLI will use these values to display the results to the user
        Ok((diagnosis_id, ai_question, session.status))
    }

    /// Get an existing diagnosis session
    ///
    /// This is a simple query operation - retrieve a diagnosis by ID.
    /// Returns Option because the diagnosis might not exist.
    ///
    /// # Arguments
    /// * `&self` - Borrows the service
    /// * `diagnosis_id` - UUID of the diagnosis to retrieve
    ///
    /// # Returns
    /// * `Ok(Some(session))` - If diagnosis found
    /// * `Ok(None)` - If diagnosis not found
    /// * `Err` - If database error occurs
    pub fn get_diagnosis(&self, diagnosis_id: Uuid) -> Result<Option<DiagnosisSession>> {
        // Simply delegate to the repository
        // The repository handles the SQL query and deserialization
        self.diagnosis_repo
            .get_by_id(diagnosis_id)
            .map_err(|e| anyhow!("Database error: {}", e))
    }
}
