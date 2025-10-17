use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use super::Message; // Reuse Message from diagnosis.rs if needed

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: Uuid,
    pub plant_id: Uuid,
    pub messages: Vec<Message>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}