use rusqlite::{Connection, Result as SqliteResult};
use uuid::Uuid;
use crate::models::Plant;
use std::sync::{Arc, Mutex};

pub struct PlantRepository {
    conn: Arc<Mutex<Connection>>,
}

impl PlantRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn get_by_id(&self, plant_id: Uuid) -> SqliteResult<Option<Plant>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, care_instructions, watering_schedule, image_path, created_at
             FROM plants WHERE id = ?1"
        )?;

        let mut rows = stmt.query([plant_id.to_string()])?;

        if let Some(row) = rows.next()? {
            let care_instructions: Option<String> = row.get(2)?;
            let watering_schedule: Option<String> = row.get(3)?;

            let care_schedule = if care_instructions.is_some() || watering_schedule.is_some() {
                Some(crate::models::CareSchedule {
                    care_instructions: care_instructions.unwrap_or_default(),
                    watering_schedule: watering_schedule.unwrap_or_default(),
                })
            } else {
                None
            };

            Ok(Some(Plant {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                name: row.get(1)?,
                care_schedule,
                image_path: row.get(4)?,
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                    .unwrap()
                    .with_timezone(&chrono::Utc),
            }))
        } else {
            Ok(None)
        }
    }
}

