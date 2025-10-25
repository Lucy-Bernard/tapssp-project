use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::Row;

use crate::config::Database;
use crate::domain::enums::DiagnosisStatus;
use crate::domain::DiagnosisSession;

#[derive(Clone)]
pub struct DiagnosisRepository {
    db: Database,
}

impl DiagnosisRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn create(&self, session: &DiagnosisSession) -> Result<DiagnosisSession> {
        let context_json = serde_json::to_string(&session.diagnosis_context)?;

        sqlx::query(
            r#"
            INSERT INTO diagnosis_sessions (id, plant_id, status, diagnosis_context, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&session.id)
        .bind(&session.plant_id)
        .bind(session.status.as_str())
        .bind(&context_json)
        .bind(session.created_at.to_rfc3339())
        .bind(session.updated_at.to_rfc3339())
        .execute(self.db.pool())
        .await?;

        Ok(session.clone())
    }

    pub async fn get_by_id(&self, id: &str) -> Result<Option<DiagnosisSession>> {
        let row = sqlx::query(
            r#"
            SELECT id, plant_id, status, diagnosis_context, created_at, updated_at
            FROM diagnosis_sessions
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(self.db.pool())
        .await?;

        match row {
            Some(row) => {
                let status_str: String = row.get("status");
                let status = DiagnosisStatus::from_str(&status_str)
                    .ok_or_else(|| anyhow::anyhow!("Invalid diagnosis status"))?;
                let context_str: String = row.get("diagnosis_context");
                let context = serde_json::from_str(&context_str)?;
                let created_at: String = row.get("created_at");
                let updated_at: String = row.get("updated_at");

                Ok(Some(DiagnosisSession {
                    id: row.get("id"),
                    plant_id: row.get("plant_id"),
                    status,
                    diagnosis_context: context,
                    created_at: DateTime::parse_from_rfc3339(&created_at)?
                        .with_timezone(&Utc),
                    updated_at: DateTime::parse_from_rfc3339(&updated_at)?
                        .with_timezone(&Utc),
                }))
            }
            None => Ok(None),
        }
    }

    pub async fn get_all_by_plant_id(
        &self,
        plant_id: &str,
        _user_id: &str,
    ) -> Result<Vec<DiagnosisSession>> {
        let rows = sqlx::query(
            r#"
            SELECT id, plant_id, status, diagnosis_context, created_at, updated_at
            FROM diagnosis_sessions
            WHERE plant_id = ?
            ORDER BY created_at DESC
            "#,
        )
        .bind(plant_id)
        .fetch_all(self.db.pool())
        .await?;

        let mut sessions = Vec::new();
        for row in rows {
            let status_str: String = row.get("status");
            let status = DiagnosisStatus::from_str(&status_str)
                .ok_or_else(|| anyhow::anyhow!("Invalid diagnosis status"))?;
            let context_str: String = row.get("diagnosis_context");
            let context = serde_json::from_str(&context_str)?;
            let created_at: String = row.get("created_at");
            let updated_at: String = row.get("updated_at");

            sessions.push(DiagnosisSession {
                id: row.get("id"),
                plant_id: row.get("plant_id"),
                status,
                diagnosis_context: context,
                created_at: DateTime::parse_from_rfc3339(&created_at)?.with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&updated_at)?.with_timezone(&Utc),
            });
        }

        Ok(sessions)
    }

    pub async fn update(&self, session: &DiagnosisSession) -> Result<()> {
        let context_json = serde_json::to_string(&session.diagnosis_context)?;

        sqlx::query(
            r#"
            UPDATE diagnosis_sessions
            SET status = ?, diagnosis_context = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(session.status.as_str())
        .bind(&context_json)
        .bind(session.updated_at.to_rfc3339())
        .bind(&session.id)
        .execute(self.db.pool())
        .await?;

        Ok(())
    }

    pub async fn delete(&self, id: &str) -> Result<()> {
        sqlx::query(
            r#"
            DELETE FROM diagnosis_sessions
            WHERE id = ?
            "#,
        )
        .bind(id)
        .execute(self.db.pool())
        .await?;

        Ok(())
    }
}