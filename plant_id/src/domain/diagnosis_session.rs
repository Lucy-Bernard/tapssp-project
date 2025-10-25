//! DIAGNOSIS SESSION DOMAIN MODEL
//!
//! Represents a diagnostic conversation session between the user and AI.
//! Contains the full context needed for the diagnostic kernel to operate.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::domain::enums::DiagnosisStatus;

/// Represents an ongoing or completed diagnosis session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosisSession {
    pub id: String,
    pub plant_id: String,
    pub status: DiagnosisStatus,
    pub diagnosis_context: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl DiagnosisSession {
    pub fn new(plant_id: String, initial_prompt: String) -> Self {
        let now = Utc::now();
        let context = serde_json::json!({
            "initial_prompt": initial_prompt,
            "conversation_history": [
                {"role": "user", "message": initial_prompt}
            ],
            "state": {},
            "plant_vitals": null
        });

        Self {
            id: Uuid::new_v4().to_string(),
            plant_id,
            status: DiagnosisStatus::PendingUserInput,
            diagnosis_context: context,
            created_at: now,
            updated_at: now,
        }
    }
}