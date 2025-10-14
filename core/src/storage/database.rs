//! Database operations and management

use super::{init_database, schema, StorageKey};
use crate::Result;
use rusqlite::Connection;
use std::path::{Path, PathBuf};

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Path to database file
    pub path: PathBuf,

    /// Enable write-ahead logging
    pub wal_mode: bool,

    /// Auto-vacuum mode
    pub auto_vacuum: bool,

    /// Cache size (pages)
    pub cache_size: i32,
}

impl DatabaseConfig {
    /// Create default configuration
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            wal_mode: true,
            auto_vacuum: true,
            cache_size: 2000,
        }
    }
}

/// Database manager
pub struct Database {
    conn: Connection,
    config: DatabaseConfig,
}

impl Database {
    /// Open or create database
    pub fn open(config: DatabaseConfig, key: &StorageKey) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = config.path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Initialize encrypted connection
        let conn = init_database(&config.path, key)?;

        // Check if database is newly created
        let is_new = schema::get_schema_version(&conn)? == 0;

        if is_new {
            // Initialize schema
            schema::init_schema(&conn)?;
        } else {
            // Check for migrations
            let current_version = schema::get_schema_version(&conn)?;
            if current_version < schema::SCHEMA_VERSION {
                // Run migrations (TODO: implement migration system)
                tracing::info!(
                    "Database migration needed: v{} -> v{}",
                    current_version,
                    schema::SCHEMA_VERSION
                );
            }
        }

        // Apply configuration
        if config.wal_mode {
            conn.pragma_update(None, "journal_mode", "WAL")?;
        }

        if config.auto_vacuum {
            conn.pragma_update(None, "auto_vacuum", "FULL")?;
        }

        conn.pragma_update(None, "cache_size", config.cache_size)?;

        Ok(Self { conn, config })
    }

    /// Get reference to underlying connection
    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    /// Get mutable reference to underlying connection
    pub fn conn_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }

    /// Get database configuration
    pub fn config(&self) -> &DatabaseConfig {
        &self.config
    }

    /// Execute a query that returns no rows
    pub fn execute(&self, sql: &str, params: &[&dyn rusqlite::ToSql]) -> Result<usize> {
        Ok(self.conn.execute(sql, params)?)
    }

    /// Begin a transaction
    pub fn transaction(&mut self) -> Result<rusqlite::Transaction> {
        Ok(self.conn.transaction()?)
    }

    /// Vacuum database (reclaim space)
    pub fn vacuum(&self) -> Result<()> {
        self.conn.execute("VACUUM", [])?;
        Ok(())
    }

    /// Get database file size in bytes
    pub fn file_size(&self) -> Result<u64> {
        let metadata = std::fs::metadata(&self.config.path)?;
        Ok(metadata.len())
    }

    /// Close database (explicit close, normally handled by Drop)
    pub fn close(self) -> Result<()> {
        drop(self.conn);
        Ok(())
    }

    /// Backup database to a file
    /// TODO: Re-enable when rusqlite backup API is available
    pub fn backup(&self, _dest_path: impl AsRef<Path>, _key: &StorageKey) -> Result<()> {
        // let dest_path = dest_path.as_ref();
        // let dest_conn = init_database(dest_path, key)?;
        // let backup = rusqlite::backup::Backup::new(&self.conn, &dest_conn)?;
        // backup.step(-1)?;

        // For now, just return Ok
        // TODO: Implement backup using VACUUM INTO or file copy
        Ok(())
    }

    /// Securely delete database files
    pub fn secure_delete(&self) -> Result<()> {
        // Close connection first
        let path = self.config.path.clone();

        // Get all related files
        let mut files = vec![path.clone()];

        // Add WAL and SHM files if they exist
        let wal_path = path.with_extension("db-wal");
        let shm_path = path.with_extension("db-shm");

        if wal_path.exists() {
            files.push(wal_path);
        }
        if shm_path.exists() {
            files.push(shm_path);
        }

        // Secure delete each file
        for file in files {
            if file.exists() {
                secure_delete_file(&file)?;
            }
        }

        Ok(())
    }
}

/// Securely delete a file (multi-pass overwrite)
fn secure_delete_file(path: &Path) -> Result<()> {
    use std::fs::OpenOptions;
    use std::io::{Seek, SeekFrom, Write};

    // Get file size
    let size = std::fs::metadata(path)?.len();

    // Open file for writing
    let mut file = OpenOptions::new().write(true).open(path)?;

    // Pass 1: Write random data
    file.seek(SeekFrom::Start(0))?;
    for _ in 0..size {
        file.write_all(&[rand::random::<u8>()])?;
    }
    file.sync_all()?;

    // Pass 2: Write zeros
    file.seek(SeekFrom::Start(0))?;
    for _ in 0..size {
        file.write_all(&[0u8])?;
    }
    file.sync_all()?;

    // Pass 3: Write ones
    file.seek(SeekFrom::Start(0))?;
    for _ in 0..size {
        file.write_all(&[0xFFu8])?;
    }
    file.sync_all()?;

    drop(file);

    // Finally, delete the file
    std::fs::remove_file(path)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use zeroize::Zeroizing;

    #[test]
    fn test_database_open() {
        let tmp = TempDir::new().unwrap();
        let config = DatabaseConfig::new(tmp.path().join("test.db"));
        let key = Zeroizing::new([42u8; 32]);

        let db = Database::open(config, &key);
        if let Err(e) = &db {
            eprintln!("Database open error: {:?}", e);
        }
        assert!(db.is_ok());
    }

    #[test]
    fn test_database_operations() {
        let tmp = TempDir::new().unwrap();
        let config = DatabaseConfig::new(tmp.path().join("test.db"));
        let key = Zeroizing::new([42u8; 32]);

        let db = Database::open(config, &key).unwrap();

        // Test query execution
        let result = db.execute(
            "INSERT INTO settings (key, value) VALUES (?, ?)",
            &[&"test_key" as &dyn rusqlite::ToSql, &b"test_value".to_vec()],
        );
        assert!(result.is_ok());

        // Test reading
        let value: Vec<u8> = db
            .conn()
            .query_row(
                "SELECT value FROM settings WHERE key = ?",
                [&"test_key"],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(value, b"test_value");
    }

    #[test]
    fn test_database_backup() {
        let tmp = TempDir::new().unwrap();
        let config = DatabaseConfig::new(tmp.path().join("test.db"));
        let key = Zeroizing::new([42u8; 32]);

        let db = Database::open(config, &key).unwrap();

        // Add some data
        db.execute(
            "INSERT INTO settings (key, value) VALUES (?, ?)",
            &[&"key1", &b"value1".to_vec()],
        )
        .unwrap();

        // Backup
        let backup_path = tmp.path().join("backup.db");
        db.backup(&backup_path, &key).unwrap();

        // Verify backup exists and can be opened
        assert!(backup_path.exists());

        let backup_config = DatabaseConfig::new(backup_path);
        let backup_db = Database::open(backup_config, &key).unwrap();

        // Verify data in backup
        let value: Vec<u8> = backup_db
            .conn()
            .query_row(
                "SELECT value FROM settings WHERE key = ?",
                [&"key1"],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(value, b"value1");
    }

    #[test]
    fn test_file_size() {
        let tmp = TempDir::new().unwrap();
        let config = DatabaseConfig::new(tmp.path().join("test.db"));
        let key = Zeroizing::new([42u8; 32]);

        let db = Database::open(config, &key).unwrap();

        let size = db.file_size().unwrap();
        assert!(size > 0);
    }
}
