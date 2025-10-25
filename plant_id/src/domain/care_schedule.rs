use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CareSchedule {
    pub light: String,
    pub water: String,
    pub humidity: String,
    pub temperature: String,
    pub care_instructions: String,
}

impl Default for CareSchedule {
    fn default() -> Self {
        Self {
            light: "Bright, indirect sunlight".to_string(),
            water: "Water when top inch of soil is dry".to_string(),
            humidity: "Moderate humidity (40-60%)".to_string(),
            temperature: "18-24°C (65-75°F)".to_string(),
            care_instructions: String::new(),
        }
    }
}

