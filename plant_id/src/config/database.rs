/*!
 * DATABASE CONNECTION MANAGEMENT
 *
 * Manages SQLite database connections and migrations.
 * This is infrastructure code that supports repositories (secondary adapters).
 */

use anyhow::{Context, Result};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::{Pool, Sqlite};
use std::str::FromStr;

#[derive(Clone)]
pub struct Database {
    pool: Pool<Sqlite>,
}

impl Database {
    /// Create a new database connection pool
    pub async fn new() -> Result<Self> {
        let database_path = std::env::var("DATABASE_PATH")
            .unwrap_or_else(|_| "plant_care.db".to_string());

        let options = SqliteConnectOptions::from_str(&format!("sqlite://{}", database_path))?
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        Ok(Self { pool })
    }

    /// Get a reference to the connection pool
    pub fn pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }

    /// Run database migrations
    pub async fn migrate(&self) -> Result<()> {
        // Create plants table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS plants (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                name TEXT NOT NULL,
                care_schedule TEXT NOT NULL,
                image_url TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create diagnosis_sessions table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS diagnosis_sessions (
                id TEXT PRIMARY KEY,
                plant_id TEXT NOT NULL,
                status TEXT NOT NULL,
                diagnosis_context TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (plant_id) REFERENCES plants(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes for better query performance
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_plants_user_id ON plants(user_id)
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_diagnosis_sessions_plant_id ON diagnosis_sessions(plant_id)
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

/// Get environment variable or return error with helpful message
pub fn get_env(key: &str) -> Result<String> {
    std::env::var(key).context(format!("Missing required environment variable: {}", key))
}
