//! Encrypted storage using SQLCipher
//!
//! All data at rest is encrypted. The database is encrypted with a key
//! derived from the user's master key (which comes from hardware-backed storage).
//!
//! Security properties:
//! - Encryption at rest (SQLCipher with AES-256)
//! - No plaintext on disk ever
//! - Oblivious access patterns where possible
//! - Secure deletion support

pub mod database;
pub mod schema;
pub mod migrations;
pub mod repository;

pub use database::{Database, DatabaseConfig};
pub use schema::*;
pub use repository::{RepositoryManager, ContactRepository, SafeHouseRepository,
                     EmergencyRepository, IntelligenceRepository, IdentityRepository};

use crate::Result;
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use zeroize::Zeroizing;

/// Storage key for encrypting the database (derived from master key)
pub type StorageKey = Zeroizing<[u8; 32]>;

/// Initialize encrypted database connection
pub fn init_database(db_path: impl AsRef<Path>, key: &StorageKey) -> Result<Connection> {
    let conn = Connection::open(db_path)?;

    // Set SQLCipher encryption key
    let key_hex = hex::encode(key.as_ref());
    conn.pragma_update(None, "key", format!("\"x'{}'\"", key_hex))?;

    // SQLCipher v4 settings (strong encryption)
    conn.pragma_update(None, "cipher_page_size", 4096)?;
    conn.pragma_update(None, "kdf_iter", 256000)?;
    conn.pragma_update(None, "cipher_hmac_algorithm", "HMAC_SHA512")?;
    conn.pragma_update(None, "cipher_kdf_algorithm", "PBKDF2_HMAC_SHA512")?;

    // Verify encryption is working
    let _: i64 = conn.query_row("SELECT count(*) FROM sqlite_master", [], |row| row.get(0))?;

    Ok(conn)
}

/// Database file paths
#[derive(Debug, Clone)]
pub struct DatabasePaths {
    /// Main database file
    pub database: PathBuf,

    /// Database journal (for WAL mode)
    pub journal: PathBuf,

    /// Backup location
    pub backup: Option<PathBuf>,
}

impl DatabasePaths {
    /// Create paths from a base directory
    pub fn from_dir(dir: impl AsRef<Path>) -> Self {
        let dir = dir.as_ref();

        Self {
            database: dir.join("underground-railroad.db"),
            journal: dir.join("underground-railroad.db-journal"),
            backup: Some(dir.join("underground-railroad.db.backup")),
        }
    }

    /// Get all paths that need secure deletion
    pub fn all_paths(&self) -> Vec<&Path> {
        let mut paths = vec![self.database.as_path(), self.journal.as_path()];

        if let Some(backup) = &self.backup {
            paths.push(backup.as_path());
        }

        paths
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_database_init() {
        let tmp = TempDir::new().unwrap();
        let db_path = tmp.path().join("test.db");
        let key = Zeroizing::new([42u8; 32]);

        let conn = init_database(&db_path, &key);
        assert!(conn.is_ok());

        let conn = conn.unwrap();

        // Verify we can create a table
        conn.execute(
            "CREATE TABLE test (id INTEGER PRIMARY KEY, data TEXT)",
            [],
        )
        .unwrap();

        // Verify we can insert data
        conn.execute("INSERT INTO test (data) VALUES (?)", ["encrypted"])
            .unwrap();

        // Verify we can read data
        let data: String = conn
            .query_row("SELECT data FROM test WHERE id = 1", [], |row| row.get(0))
            .unwrap();

        assert_eq!(data, "encrypted");
    }

    #[test]
    fn test_wrong_key_fails() {
        let tmp = TempDir::new().unwrap();
        let db_path = tmp.path().join("test.db");

        // Create database with one key
        let key1 = Zeroizing::new([1u8; 32]);
        let conn1 = init_database(&db_path, &key1).unwrap();
        conn1
            .execute("CREATE TABLE test (id INTEGER PRIMARY KEY)", [])
            .unwrap();
        drop(conn1);

        // Try to open with different key - should fail
        let key2 = Zeroizing::new([2u8; 32]);
        let conn2 = init_database(&db_path, &key2);

        // SQLCipher will return an error when trying to read with wrong key
        assert!(conn2.is_ok()); // Connection opens, but...
        let conn2 = conn2.unwrap();
        let result = conn2.execute("SELECT * FROM test", []);
        assert!(result.is_err()); // ...queries fail
    }

    #[test]
    fn test_database_paths() {
        let tmp = TempDir::new().unwrap();
        let paths = DatabasePaths::from_dir(tmp.path());

        assert!(paths.database.ends_with("underground-railroad.db"));
        assert!(paths.journal.ends_with("underground-railroad.db-journal"));
        assert!(paths.backup.is_some());

        let all = paths.all_paths();
        assert!(all.len() >= 2);
    }
}
