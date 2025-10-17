// Diagnosis Controller - Handles diagnosis-related CLI endpoints
//
// This controller implements the CLI commands for diagnosis operations.
// In a typical MVC pattern, controllers handle user input and orchestrate
// between services and views (in our case, CLI output).
//
// Endpoints:
// - diagnose: Create a new diagnosis session
// - show-diagnosis: Display an existing diagnosis session

use anyhow::Result;
use uuid::Uuid;
use crate::service::DiagnosisService;
use crate::models::DiagnosisStatus;

/// Handle the diagnose command (endpoint)
/// 
/// This function implements the CLI command for creating a new diagnosis session.
/// It takes a plant ID and a user's problem description (prompt), then:
/// 1. Validates the plant exists
/// 2. Creates a diagnosis session in the database
/// 3. Runs the AI kernel to generate the first follow-up question
/// 4. Displays the results to the user
/// 
/// # Arguments
/// * `service` - The DiagnosisService that handles business logic and database operations
/// * `plant_id` - UUID of the plant to diagnose
/// * `prompt` - User's initial description of the problem (e.g., "leaves are browning")
/// 
/// # Returns
/// * `Ok(())` if successful
/// * `Err` if plant not found or other error occurs
pub async fn diagnose(
    service: &DiagnosisService,
    plant_id: Uuid,
    prompt: String,
) -> Result<()> {
    // Display a friendly header to the user
    println!("\n🔍 Starting diagnosis for plant {}...\n", plant_id);

    // Call the service layer to create the diagnosis session
    // This does the heavy lifting:
    // - Checks if the plant exists (returns error if not found)
    // - Creates a new DiagnosisSession with a unique ID
    // - Runs the KernelExecutor to generate an AI response
    // - Saves everything to the database
    // The service returns a tuple of (diagnosis_id, ai_question, status)
    let (diagnosis_id, ai_question, status) = service
        .create_diagnosis(plant_id, prompt.clone())
        .await?;

    // Display the results to the user
    println!("✅ Diagnosis session created!");
    println!("   Diagnosis ID: {}", diagnosis_id);

    // Convert the DiagnosisStatus enum to a string for display
    // This matches the status format expected (e.g., "PENDING_USER_INPUT")
    // We use a match expression to ensure we handle all possible status values
    let status_str = match status {
        DiagnosisStatus::PendingUserInput => "PENDING_USER_INPUT",
        DiagnosisStatus::Processing => "PROCESSING",
        DiagnosisStatus::Completed => "COMPLETED",
        DiagnosisStatus::Cancelled => "CANCELLED",
    };
    println!("   Status: {}\n", status_str);

    // Display the AI's first question to the user
    // This is the kernel's response asking for more information about the problem
    println!("🤖 AI Response:");
    println!("   {}\n", ai_question);

    Ok(())
}

/// Display a diagnosis session (endpoint)
/// 
/// This function retrieves and displays the details of an existing diagnosis session.
/// It's useful for viewing the conversation history and current status of a diagnosis.
/// 
/// # Arguments
/// * `service` - The DiagnosisService for database access
/// * `diagnosis_id` - UUID of the diagnosis session to display
/// 
/// # Returns
/// * `Ok(())` if successful (even if diagnosis not found)
/// * `Err` if database error occurs
pub fn show_diagnosis(
    service: &DiagnosisService,
    diagnosis_id: Uuid,
) -> Result<()> {
    // Query the database for the diagnosis session
    // Returns Option<DiagnosisSession> - Some if found, None if not found
    let session = service.get_diagnosis(diagnosis_id)?;

    // Pattern match on the Option to handle both cases
    match session {
        Some(diag) => {
            // Session found - display all the details
            
            // Display basic session information
            println!("\n📋 Diagnosis Session: {}", diag.id);
            println!("   Plant ID: {}", diag.plant_id);
            // Using Debug format {:?} for the enum - shows "PendingUserInput" etc.
            println!("   Status: {:?}", diag.status);
            println!("   Created: {}", diag.created_at);
            println!("   Updated: {}\n", diag.updated_at);

            // Display the conversation history
            // This shows all messages exchanged between the user and AI
            println!("💬 Conversation:");
            
            // Iterate through each message in the conversation history
            // We use a reference (&) to avoid taking ownership of the messages
            for msg in &diag.context.conversation_history {
                // Convert the MessageRole enum to a friendly display string
                // This helps distinguish between user messages and AI responses
                let role = match msg.role {
                    crate::models::MessageRole::User => "👤 User",
                    crate::models::MessageRole::Assistant => "🤖 AI",
                };
                
                // Display each message with:
                // - Role indicator (👤 User or 🤖 AI)
                // - Timestamp formatted as HH:MM:SS
                // - The actual message content
                println!("   {} [{}]: {}", role, msg.timestamp.format("%H:%M:%S"), msg.content);
            }
            println!();
        }
        None => {
            // Session not found - display a friendly error message
            // This is not a fatal error, just informational
            println!("❌ Diagnosis session {} not found", diagnosis_id);
        }
    }

    Ok(())
}

