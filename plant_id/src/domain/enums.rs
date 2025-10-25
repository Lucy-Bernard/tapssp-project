/*!
 * DOMAIN ENUMERATIONS
 *
 * Defines the core business enumerations used throughout the domain.
 */

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DiagnosisStatus {
    PendingUserInput,
    Completed,
    Cancelled,
}

impl DiagnosisStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PendingUserInput => "PENDING_USER_INPUT",
            Self::Completed => "COMPLETED",
            Self::Cancelled => "CANCELLED",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "PENDING_USER_INPUT" => Some(Self::PendingUserInput),
            "COMPLETED" => Some(Self::Completed),
            "CANCELLED" => Some(Self::Cancelled),
            _ => None,
        }
    }
}

/// Actions that can be taken during diagnosis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosisAction {
    GetPlantVitals,
    LogState,
    AskUser,
    Conclude,
}

impl DiagnosisAction {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "GET_PLANT_VITALS" => Some(Self::GetPlantVitals),
            "LOG_STATE" => Some(Self::LogState),
            "ASK_USER" => Some(Self::AskUser),
            "CONCLUDE" => Some(Self::Conclude),
            _ => None,
        }
    }
}
