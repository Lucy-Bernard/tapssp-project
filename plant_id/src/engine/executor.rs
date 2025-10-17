// This module contains the KernelExecutor, which is the "brain" of the diagnosis system.
// The kernel is responsible for:
// 1. Processing user input
// 2. Generating AI responses (questions or diagnoses)
// 3. Managing the conversation flow
// 
// In a production system, this would integrate with an AI API like OpenRouter.
// For now, it uses a simple rule-based system for demonstration.

use crate::models::{DiagnosisSession, DiagnosisStatus, Message, MessageRole};
use anyhow::Result;
use chrono::Utc;

/// KernelExecutor - The AI diagnosis engine
/// 
/// This struct represents the core AI logic of the system.
/// Currently it's a simple rule-based system, but it demonstrates
/// the interface that would be used with a real AI API.
pub struct KernelExecutor {
    // In a real implementation, this would hold:
    // - An HTTP client for calling the AI API
    // - API keys/credentials
    // - Configuration for the AI model
}

impl KernelExecutor {
    /// Create a new KernelExecutor
    /// 
    /// Currently no initialization needed, but in production this would:
    /// - Set up the HTTP client
    /// - Load API credentials
    /// - Configure the AI model parameters
    pub fn new() -> Self {
        Self {}
    }

    /// Runs the first cycle of the kernel loop
    /// 
    /// This is the main entry point for diagnosis. It takes the user's initial
    /// problem description and generates the AI's first follow-up question.
    /// 
    /// The kernel loop works like this:
    /// 1. User provides initial symptom → Kernel asks clarifying question
    /// 2. User answers → Kernel asks another question OR provides diagnosis
    /// 3. Repeat until diagnosis is complete
    /// 
    /// This function handles step 1.
    /// 
    /// # Arguments
    /// * `&self` - Borrows the executor
    /// * `session` - Mutable reference to the diagnosis session
    ///              We need &mut because we'll modify the session by adding messages
    /// 
    /// # Returns
    /// * `Ok(String)` - The AI's generated question
    /// * `Err` - If generation fails (in production, if API call fails)
    pub async fn run_initial_cycle(&self, session: &mut DiagnosisSession) -> Result<String> {
        // STEP 1: Record the user's message in the conversation history
        // We create a Message struct to represent what the user said
        let user_message = Message {
            role: MessageRole::User,  // This message is from the user
            content: session.context.initial_prompt.clone(),  // The problem description
            timestamp: Utc::now(),    // When they sent it
        };
        
        // Add it to the session's conversation history
        // This demonstrates Rust's ownership - we move the message into the Vec
        session.context.conversation_history.push(user_message);

        // STEP 2: Generate the AI's follow-up question
        // In production, this would:
        // - Send the conversation history to OpenRouter API
        // - Get back an AI-generated response
        // - Parse the response
        // 
        // For now, we use a simple rule-based system
        let ai_question = self.generate_followup_question(&session.context.initial_prompt);

        // STEP 3: Record the AI's message in the conversation history
        let ai_message = Message {
            role: MessageRole::Assistant,  // This message is from the AI
            content: ai_question.clone(),   // The follow-up question
            timestamp: Utc::now(),          // When it was generated
        };
        
        // Add it to the conversation history
        session.context.conversation_history.push(ai_message);

        // STEP 4: Update the session status
        // Now that we've asked a question, we're waiting for the user to respond
        session.status = DiagnosisStatus::PendingUserInput;
        session.updated_at = Utc::now();

        // STEP 5: Return the AI's question so it can be displayed to the user
        Ok(ai_question)
    }

    /// Generate a contextual follow-up question based on the initial prompt
    /// 
    /// This is a simple rule-based system for demonstration purposes.
    /// It looks for keywords in the user's prompt and generates an appropriate question.
    /// 
    /// In production, this would be replaced with an actual AI API call.
    /// 
    /// # Why rule-based for now?
    /// - Demonstrates the interface without requiring API keys
    /// - Shows the kind of logic the AI would perform
    /// - Makes testing easier
    /// 
    /// # Arguments
    /// * `initial_prompt` - The user's problem description
    /// 
    /// # Returns
    /// * `String` - A contextual follow-up question
    fn generate_followup_question(&self, initial_prompt: &str) -> String {
        // Convert to lowercase for case-insensitive matching
        let prompt_lower = initial_prompt.to_lowercase();
        
        // Simple keyword matching to generate contextual questions
        // Each branch handles a different category of plant problem
        
        if prompt_lower.contains("brown") || prompt_lower.contains("yellow") {
            // Discoloration issues - often related to light or nutrients
            "To begin, are the brown spots mostly on the leaves getting the most sun?".to_string()
        } else if prompt_lower.contains("wilting") || prompt_lower.contains("drooping") {
            // Structural issues - often related to watering
            "Can you tell me how often you've been watering this plant?".to_string()
        } else if prompt_lower.contains("spots") || prompt_lower.contains("dot") {
            // Spots could be disease, pests, or nutrient issues
            "What color are these spots, and are they raised or flat?".to_string()
        } else if prompt_lower.contains("pest") || prompt_lower.contains("bug") {
            // Pest identification requires visual description
            "Can you describe the pests? Are they small dots, flying insects, or something else?".to_string()
        } else {
            // Generic fallback for unrecognized symptoms
            "Could you provide more details about the symptoms you're observing? For example, which parts of the plant are affected?".to_string()
        }
    }
}

/// Implement the Default trait for KernelExecutor
/// 
/// This allows creating a KernelExecutor with `KernelExecutor::default()`
/// It's a common Rust pattern for types with simple initialization.
impl Default for KernelExecutor {
    fn default() -> Self {
        Self::new()
    }
}
