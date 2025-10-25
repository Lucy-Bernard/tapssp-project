/*!
 * DATA TRANSFER OBJECTS (DTOs)
 *
 * Structures used to transfer data between layers and external systems.
 */

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlantCreationDto {
    pub images: Vec<String>, // Base64 encoded images
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosisStartDto {
    pub prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosisUpdateDto {
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DiagnosisResponseDto {
    #[serde(rename = "ask")]
    Ask(DiagnosisAskResponse),
    #[serde(rename = "conclude")]
    Conclude(DiagnosisConcludeResponse),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosisAskResponse {
    pub diagnosis_id: String,
    pub question: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosisConcludeResponse {
    pub diagnosis_id: String,
    pub finding: String,
    pub recommendation: String,
}

