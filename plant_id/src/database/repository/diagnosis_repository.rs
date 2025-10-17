// This module implements the Repository Pattern for diagnosis data.
//
// The Repository Pattern is a design pattern that:
// - Encapsulates database access logic
// - Provides a clean API for data operations
// - Makes it easy to swap out the database implementation
// - Keeps SQL queries separate from business logic
//
// This demonstrates Rust's approach to managing shared mutable state with Arc<Mutex<T>>

use rusqlite::{Connection, Result as SqliteResult};
use uuid::Uuid;
use crate::models::{DiagnosisSession, DiagnosisStatus, DiagnosisContext};
use std::sync::{Arc, Mutex};
use chrono::Utc;

/// DiagnosisRepository - Handles all database operations for diagnoses
///
/// Why Arc<Mutex<Connection>>?
/// - Arc (Atomic Reference Counting) allows multiple owners of the connection
/// - Mutex ensures only one thread can access the connection at a time
/// - This is Rust's way of providing thread-safe shared access
///
/// This demonstrates:
/// - Ownership: The Arc shares ownership of the Connection
/// - Borrowing: We borrow the connection when we need it
/// - Thread safety: Mutex prevents data races
pub struct DiagnosisRepository {
    conn: Arc<Mutex<Connection>>,  // Shared, thread-safe database connection
}

impl DiagnosisRepository {
    /// Create a new repository with a database connection
    ///
    /// We take an Arc<Mutex<Connection>> so multiple repositories can share
    /// the same connection. This is more efficient than creating a new
    /// connection for each repository.
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Save a new diagnosis session to the database
    ///
    /// This method:
    /// 1. Converts the DiagnosisSession struct into SQL-compatible data
    /// 2. Executes an INSERT statement
    /// 3. Returns Result for error handling
    ///
    /// # Arguments
    /// * `&self` - Borrows the repository
    /// * `session` - Borrows the diagnosis session (we don't need to own it)
    ///
    /// # Returns
    /// * `Ok(())` - Success
    /// * `Err` - If the database operation fails
    pub fn create(&self, session: &DiagnosisSession) -> SqliteResult<()> {
        // STEP 1: Acquire the lock on the database connection
        // .lock() blocks until we can get exclusive access
        // .unwrap() panics if the lock is poisoned (thread panicked while holding it)
        let conn = self.conn.lock().unwrap();

        // STEP 2: Convert the DiagnosisStatus enum to a string for the database
        // SQLite doesn't have an enum type, so we store it as TEXT
        // This match ensures we handle all possible status values
        let status_str = match session.status {
            DiagnosisStatus::PendingUserInput => "PENDING_USER_INPUT",
            DiagnosisStatus::Processing => "PROCESSING",
            DiagnosisStatus::Completed => "COMPLETED",
            DiagnosisStatus::Cancelled => "CANCELLED",
        };

        // STEP 3: Serialize the context to JSON
        // The DiagnosisContext contains complex data (HashMap, Vec of Messages)
        // We store it as a JSON string in the database
        // .unwrap() is safe here because DiagnosisContext always serializes successfully
        let context_json = serde_json::to_string(&session.context).unwrap();

        // STEP 4: Execute the INSERT statement
        // ? placeholders prevent SQL injection
        // We pass a tuple of values that will replace the ? placeholders
        conn.execute(
            "INSERT INTO diagnoses (id, plant_id, status, context, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (
                session.id.to_string(),          // ?1 - Convert UUID to string
                session.plant_id.to_string(),     // ?2 - Convert UUID to string
                status_str,                       // ?3 - The status string
                context_json,                     // ?4 - The serialized context
                session.created_at.to_rfc3339(),  // ?5 - ISO 8601 timestamp
                session.updated_at.to_rfc3339(),  // ?6 - ISO 8601 timestamp
            ),
        )?;

        // The ? operator propagates any SQL error up to the caller
        // If we reach here, the insert was successful
        Ok(())
    }

    /// Retrieve a diagnosis session from the database by ID
    ///
    /// This method:
    /// 1. Queries the database for a diagnosis with the given ID
    /// 2. Converts the database row back into a DiagnosisSession struct
    /// 3. Returns Option<DiagnosisSession> - Some if found, None if not
    ///
    /// This demonstrates:
    /// - SQL queries with parameters
    /// - Converting database types to Rust types
    /// - Deserializing JSON back into structs
    ///
    /// # Arguments
    /// * `&self` - Borrows the repository
    /// * `diagnosis_id` - The UUID of the diagnosis to find
    ///
    /// # Returns
    /// * `Ok(Some(session))` - Diagnosis found
    /// * `Ok(None)` - Diagnosis not found
    /// * `Err` - Database error
    pub fn get_by_id(&self, diagnosis_id: Uuid) -> SqliteResult<Option<DiagnosisSession>> {
        // STEP 1: Acquire the database lock
        let conn = self.conn.lock().unwrap();

        // STEP 2: Prepare the SQL query
        // Using a prepared statement is more efficient and prevents SQL injection
        let mut stmt = conn.prepare(
            "SELECT id, plant_id, status, context, created_at, updated_at
             FROM diagnoses WHERE id = ?1"
        )?;

        // STEP 3: Execute the query with our parameter
        // .query() returns an iterator over the result rows
        let mut rows = stmt.query([diagnosis_id.to_string()])?;

        // STEP 4: Check if we got a row
        // .next()? gets the next row, or None if no rows found
        if let Some(row) = rows.next()? {
            // We found a row! Now reconstruct the DiagnosisSession

            // Get the status string and convert it back to the enum
            let status_str: String = row.get(2)?;
            let status = match status_str.as_str() {
                "PENDING_USER_INPUT" => DiagnosisStatus::PendingUserInput,
                "PROCESSING" => DiagnosisStatus::Processing,
                "COMPLETED" => DiagnosisStatus::Completed,
                "CANCELLED" => DiagnosisStatus::Cancelled,
                _ => DiagnosisStatus::PendingUserInput,  // Fallback for unknown status
            };

            // Get the JSON context and deserialize it back to DiagnosisContext
            let context_json: String = row.get(3)?;
            let context: DiagnosisContext = serde_json::from_str(&context_json).unwrap();

            // Build and return the DiagnosisSession struct
            Ok(Some(DiagnosisSession {
                // row.get(0) gets the first column (id)
                // We parse it from String back to Uuid
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                plant_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap(),
                status,
                context,
                // Parse the RFC3339 timestamp back to DateTime<Utc>
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                    .unwrap()
                    .with_timezone(&Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                    .unwrap()
                    .with_timezone(&Utc),
            }))
        } else {
            // No row found - return None
            Ok(None)
        }
    }
}
