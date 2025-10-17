use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct DiagnosisSession {
    pub id: Uuid,
    pub plant_id: Uuid,
    pub status: DiagnosisStatus,
    pub context: DiagnosisContext,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DiagnosisStatus {
    PendingUserInput,
    Processing,
    Completed,
    Cancelled,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiagnosisContext {
    pub initial_prompt: String,
    pub conversation_history: Vec<Message>,
    pub state: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MessageRole {
    User,
    Assistant,
}