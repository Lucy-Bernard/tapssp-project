/*
 * PLANT ID ADAPTER
 *
 * Secondary adapter for plant identification using PlantID API.
 */

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::get_env;
use crate::dto::PlantCreationDto;

pub struct PlantIdAdapter {
    client: Client,
    api_key: String,
}

#[derive(Debug, Serialize)]
struct IdentificationRequest {
    images: Vec<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct IdentificationResponse {
    suggestions: Vec<Suggestion>,
}

#[derive(Debug, Deserialize)]
struct Suggestion {
    plant_name: String,
}

impl PlantIdAdapter {
    pub fn new() -> Result<Self> {
        let api_key = get_env("PLANT_ID_API_KEY")?;

        Ok(Self {
            client: Client::new(),
            api_key,
        })
    }

    pub async fn identify_plant(&self, dto: &PlantCreationDto) -> Result<String> {
        let request = IdentificationRequest {
            images: dto.images.clone(),
            latitude: dto.latitude,
            longitude: dto.longitude,
        };

        let response = self
            .client
            .post("https://api.plant.id/v2/identify")
            .header("Api-Key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("PlantID API error: {}", error_text);
        }

        let identification: IdentificationResponse = response.json().await?;

        let plant_name = identification
            .suggestions
            .first()
            .map(|s| s.plant_name.clone())
            .context("No plant suggestions returned from PlantID API")?;

        Ok(plant_name)
    }
}
