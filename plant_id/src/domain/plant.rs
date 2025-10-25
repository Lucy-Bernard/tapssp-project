use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::care_schedule::CareSchedule;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plant {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub care_schedule: CareSchedule,
    pub image_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Plant {
    pub fn new(user_id: String, name: String, care_schedule: CareSchedule) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            name,
            care_schedule,
            image_url: None,
            created_at: now,
            updated_at: now,
        }
    }
}

