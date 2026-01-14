use crate::models::CommandRecord;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::path::PathBuf;

pub struct Storage {
    conn: Connection,
}

impl Storage {
    /// Create a new storage instance with the given database path
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let conn = Connection::open(&db_path)
            .with_context(|| format!("Failed to open database at {:?}", db_path))?;

        let storage = Self { conn };
        storage.initialize()?;
        Ok(storage)
    }

    /// Initialize the database schema
    fn initialize(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS commands (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                command TEXT NOT NULL,
                exit_code INTEGER NOT NULL,
                duration_ms INTEGER NOT NULL,
                timestamp TEXT NOT NULL,
                working_directory TEXT NOT NULL,
                output TEXT,
                annotation TEXT
            )",
            [],
        )?;

        // Create indexes for common queries
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_timestamp ON commands(timestamp DESC)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_exit_code ON commands(exit_code)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_working_directory ON commands(working_directory)",
            [],
        )?;

        Ok(())
    }

    /// Insert a new command record
    pub fn insert(&self, record: &CommandRecord) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO commands (command, exit_code, duration_ms, timestamp, working_directory, output, annotation)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                record.command,
                record.exit_code,
                record.duration_ms,
                record.timestamp.to_rfc3339(),
                record.working_directory,
                record.output,
                record.annotation,
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Get a command record by ID
    pub fn get(&self, id: i64) -> Result<Option<CommandRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, command, exit_code, duration_ms, timestamp, working_directory, output, annotation
             FROM commands WHERE id = ?1",
        )?;

        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_record(row)?))
        } else {
            Ok(None)
        }
    }

    /// Get all command records, ordered by timestamp (most recent first)
    pub fn get_all(&self) -> Result<Vec<CommandRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, command, exit_code, duration_ms, timestamp, working_directory, output, annotation
             FROM commands ORDER BY timestamp DESC",
        )?;

        let rows = stmt.query_map([], |row| Self::row_to_record(row))?;

        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }

        Ok(records)
    }

    /// Get the most recent N command records
    pub fn get_recent(&self, limit: usize) -> Result<Vec<CommandRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, command, exit_code, duration_ms, timestamp, working_directory, output, annotation
             FROM commands ORDER BY timestamp DESC LIMIT ?1",
        )?;

        let rows = stmt.query_map(params![limit], |row| Self::row_to_record(row))?;

        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }

        Ok(records)
    }

    /// Update the annotation for a command record
    pub fn update_annotation(&self, id: i64, annotation: Option<String>) -> Result<()> {
        self.conn.execute(
            "UPDATE commands SET annotation = ?1 WHERE id = ?2",
            params![annotation, id],
        )?;
        Ok(())
    }

    /// Delete a command record
    pub fn delete(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM commands WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Get the total count of command records
    pub fn count(&self) -> Result<usize> {
        let count: usize = self
            .conn
            .query_row("SELECT COUNT(*) FROM commands", [], |row| row.get(0))?;
        Ok(count)
    }

    /// Helper function to convert a database row to a CommandRecord
    fn row_to_record(row: &rusqlite::Row) -> rusqlite::Result<CommandRecord> {
        let timestamp_str: String = row.get(4)?;
        let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        Ok(CommandRecord {
            id: Some(row.get(0)?),
            command: row.get(1)?,
            exit_code: row.get(2)?,
            duration_ms: row.get(3)?,
            timestamp,
            working_directory: row.get(5)?,
            output: row.get(6)?,
            annotation: row.get(7)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_storage_lifecycle() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let storage = Storage::new(temp_file.path().to_path_buf())?;

        // Insert a record
        let record = CommandRecord::new(
            "ls -la".to_string(),
            0,
            150,
            "/tmp".to_string(),
            Some("output".to_string()),
        );

        let id = storage.insert(&record)?;
        assert!(id > 0);

        // Retrieve the record
        let retrieved = storage.get(id)?.unwrap();
        assert_eq!(retrieved.command, "ls -la");
        assert_eq!(retrieved.exit_code, 0);
        assert_eq!(retrieved.duration_ms, 150);

        // Update annotation
        storage.update_annotation(id, Some("test annotation".to_string()))?;
        let updated = storage.get(id)?.unwrap();
        assert_eq!(updated.annotation, Some("test annotation".to_string()));

        // Count
        assert_eq!(storage.count()?, 1);

        // Delete
        storage.delete(id)?;
        assert_eq!(storage.count()?, 0);

        Ok(())
    }
}
