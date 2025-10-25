/*!
 * PLANT SERVICE
 *
 * Business logic for plant management operations.
 */

use anyhow::{Context, Result};

use crate::adapters::{AiAdapter, PlantIdAdapter, StorageAdapter};
use crate::domain::Plant;
use crate::dto::PlantCreationDto;
use crate::repositories::PlantRepository;

pub struct PlantService {
    plant_repo: PlantRepository,
    plant_id_adapter: PlantIdAdapter,
    ai_adapter: AiAdapter,
    storage_adapter: StorageAdapter,
}

impl PlantService {
    pub fn new(
        plant_repo: PlantRepository,
        plant_id_adapter: PlantIdAdapter,
        ai_adapter: AiAdapter,
        storage_adapter: StorageAdapter,
    ) -> Self {
        Self {
            plant_repo,
            plant_id_adapter,
            ai_adapter,
            storage_adapter,
        }
    }

    pub async fn create_plant(&self, dto: PlantCreationDto, user_id: String) -> Result<Plant> {
        // Step 1: Identify plant from image
        let plant_name = self
            .plant_id_adapter
            .identify_plant(&dto)
            .await
            .context("Failed to identify plant")?;

        // Step 2: Generate AI care schedule
        let care_schedule = self
            .ai_adapter
            .generate_care_schedule(&plant_name)
            .await
            .context("Failed to generate care schedule")?;

        // Step 3: Save image (decode from base64 and store locally)
        let image_url = if let Some(base64_image) = dto.images.first() {
            let image_data = base64::decode(base64_image)
                .context("Failed to decode base64 image")?;

            let filename = format!("{}.jpg", uuid::Uuid::new_v4());
            Some(
                self.storage_adapter
                    .upload_image(&image_data, &filename)
                    .await?,
            )
        } else {
            None
        };

        // Step 4: Create and save plant
        let mut plant = Plant::new(user_id, plant_name, care_schedule);
        plant.image_url = image_url;

        let plant = self.plant_repo.create(&plant).await?;

        Ok(plant)
    }
}
