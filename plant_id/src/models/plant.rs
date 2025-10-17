use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Plant {
    pub id: Uuid,
    pub name: String,
    pub care_schedule: Option<CareSchedule>,
    pub image_path: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CareSchedule {
    pub care_instructions: String,
    pub watering_schedule: String,
}