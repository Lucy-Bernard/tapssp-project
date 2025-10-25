use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::Row;

use crate::config::Database;
use crate::domain::{CareSchedule, Plant};

#[derive(Clone)]
pub struct PlantRepository {
    db: Database,
}

impl PlantRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn create(&self, plant: &Plant) -> Result<Plant> {
        let care_schedule_json = serde_json::to_string(&plant.care_schedule)?;

        sqlx::query(
            r#"
            INSERT INTO plants (id, user_id, name, care_schedule, image_url, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&plant.id)
        .bind(&plant.user_id)
        .bind(&plant.name)
        .bind(&care_schedule_json)
        .bind(&plant.image_url)
        .bind(plant.created_at.to_rfc3339())
        .bind(plant.updated_at.to_rfc3339())
        .execute(self.db.pool())
        .await?;

        Ok(plant.clone())
    }

    pub async fn get_by_id(&self, id: &str, user_id: &str) -> Result<Option<Plant>> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, name, care_schedule, image_url, created_at, updated_at
            FROM plants
            WHERE id = ? AND user_id = ?
            "#,
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(self.db.pool())
        .await?;

        match row {
            Some(row) => {
                let care_schedule: CareSchedule =
                    serde_json::from_str(row.get("care_schedule"))?;
                let created_at: String = row.get("created_at");
                let updated_at: String = row.get("updated_at");

                Ok(Some(Plant {
                    id: row.get("id"),
                    user_id: row.get("user_id"),
                    name: row.get("name"),
                    care_schedule,
                    image_url: row.get("image_url"),
                    created_at: DateTime::parse_from_rfc3339(&created_at)?
                        .with_timezone(&Utc),
                    updated_at: DateTime::parse_from_rfc3339(&updated_at)?
                        .with_timezone(&Utc),
                }))
            }
            None => Ok(None),
        }
    }

    pub async fn get_all_by_user(&self, user_id: &str) -> Result<Vec<Plant>> {
        let rows = sqlx::query(
            r#"
            SELECT id, user_id, name, care_schedule, image_url, created_at, updated_at
            FROM plants
            WHERE user_id = ?
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(self.db.pool())
        .await?;

        let mut plants = Vec::new();
        for row in rows {
            let care_schedule: CareSchedule = serde_json::from_str(row.get("care_schedule"))?;
            let created_at: String = row.get("created_at");
            let updated_at: String = row.get("updated_at");

            plants.push(Plant {
                id: row.get("id"),
                user_id: row.get("user_id"),
                name: row.get("name"),
                care_schedule,
                image_url: row.get("image_url"),
                created_at: DateTime::parse_from_rfc3339(&created_at)?.with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&updated_at)?.with_timezone(&Utc),
            });
        }

        Ok(plants)
    }

    pub async fn delete(&self, id: &str, user_id: &str) -> Result<()> {
        sqlx::query(
            r#"
            DELETE FROM plants
            WHERE id = ? AND user_id = ?
            "#,
        )
        .bind(id)
        .bind(user_id)
        .execute(self.db.pool())
        .await?;

        Ok(())
    }

    pub async fn update(&self, plant: &Plant) -> Result<()> {
        let care_schedule_json = serde_json::to_string(&plant.care_schedule)?;

        sqlx::query(
            r#"
            UPDATE plants
            SET name = ?, care_schedule = ?, image_url = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&plant.name)
        .bind(&care_schedule_json)
        .bind(&plant.image_url)
        .bind(plant.updated_at.to_rfc3339())
        .bind(&plant.id)
        .execute(self.db.pool())
        .await?;

        Ok(())
    }
}
