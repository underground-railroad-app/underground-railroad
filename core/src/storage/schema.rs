//! Database schema definitions
//!
//! All tables are automatically encrypted by SQLCipher.
//! Design considerations:
//! - Minimal metadata (no timestamps that reveal patterns)
//! - BLOB storage for encrypted data
//! - Indexes only where necessary (prevent metadata leakage)

use crate::Result;
use rusqlite::Connection;

/// Current schema version
pub const SCHEMA_VERSION: i32 = 1;

/// Initialize database schema
pub fn init_schema(conn: &Connection) -> Result<()> {
    // Enable WAL mode for better concurrency
    conn.pragma_update(None, "journal_mode", "WAL")?;

    // Foreign keys
    conn.pragma_update(None, "foreign_keys", "ON")?;

    // Create all tables
    create_identity_table(conn)?;
    create_contacts_table(conn)?;
    create_safe_houses_table(conn)?;
    create_transport_table(conn)?;
    create_emergencies_table(conn)?;
    create_intelligence_table(conn)?;
    create_messages_table(conn)?;
    create_trust_table(conn)?;
    create_settings_table(conn)?;
    create_metadata_table(conn)?;

    // Set schema version
    set_schema_version(conn, SCHEMA_VERSION)?;

    Ok(())
}

/// Identity table - stores user's own identity/persona
fn create_identity_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS identity (
            id BLOB PRIMARY KEY,
            name TEXT NOT NULL,
            fingerprint BLOB NOT NULL,
            veilid_dht_key BLOB NOT NULL,
            veilid_route BLOB NOT NULL,
            keypair_encrypted BLOB NOT NULL,
            created_at INTEGER NOT NULL,
            is_primary BOOLEAN NOT NULL DEFAULT 0
        )",
        [],
    )?;

    Ok(())
}

/// Contacts table - people in your trust network
fn create_contacts_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS contacts (
            id BLOB PRIMARY KEY,
            name TEXT NOT NULL,
            fingerprint BLOB NOT NULL,
            veilid_route BLOB NOT NULL,
            trust_level INTEGER NOT NULL,
            verification_method INTEGER NOT NULL,
            region TEXT,
            languages TEXT,
            capabilities TEXT,
            tags TEXT,
            added_at INTEGER NOT NULL,
            last_contact INTEGER,
            introduced_by BLOB,
            notes BLOB,
            available BOOLEAN NOT NULL DEFAULT 1,
            FOREIGN KEY (introduced_by) REFERENCES contacts(id)
        )",
        [],
    )?;

    // Index for trust lookups
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_contacts_trust
         ON contacts(trust_level)",
        [],
    )?;

    Ok(())
}

/// Safe houses table
fn create_safe_houses_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS safe_houses (
            id BLOB PRIMARY KEY,
            operator_id BLOB NOT NULL,
            name TEXT NOT NULL,
            region TEXT NOT NULL,
            capabilities TEXT NOT NULL,
            capacity INTEGER NOT NULL,
            current_occupancy INTEGER NOT NULL DEFAULT 0,
            status INTEGER NOT NULL,
            accommodations TEXT,
            max_stay_days INTEGER,
            notes BLOB,
            verified BOOLEAN NOT NULL DEFAULT 0,
            registered_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            FOREIGN KEY (operator_id) REFERENCES identity(id)
        )",
        [],
    )?;

    // Index for availability queries
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_safe_houses_status
         ON safe_houses(status, current_occupancy)",
        [],
    )?;

    Ok(())
}

/// Transportation table (both offers and requests)
fn create_transport_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS transportation (
            id BLOB PRIMARY KEY,
            type INTEGER NOT NULL,
            person_id BLOB NOT NULL,
            from_region TEXT,
            to_region TEXT,
            vehicle_type INTEGER,
            capacity INTEGER,
            num_people INTEGER,
            capabilities TEXT,
            requirements TEXT,
            status INTEGER NOT NULL,
            availability INTEGER,
            timing INTEGER,
            notes BLOB,
            created_at INTEGER NOT NULL,
            expires_at INTEGER NOT NULL,
            FOREIGN KEY (person_id) REFERENCES contacts(id)
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_transport_status
         ON transportation(status, type)",
        [],
    )?;

    Ok(())
}

/// Emergency requests table
fn create_emergencies_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS emergencies (
            id BLOB PRIMARY KEY,
            requester_id BLOB,
            needs TEXT NOT NULL,
            region TEXT NOT NULL,
            urgency INTEGER NOT NULL,
            num_people INTEGER NOT NULL,
            num_children INTEGER NOT NULL DEFAULT 0,
            special_needs BLOB,
            notes BLOB,
            status INTEGER NOT NULL,
            created_at INTEGER NOT NULL,
            expires_at INTEGER NOT NULL,
            FOREIGN KEY (requester_id) REFERENCES contacts(id)
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_emergencies_status
         ON emergencies(status, urgency)",
        [],
    )?;

    Ok(())
}

/// Intelligence reports table
fn create_intelligence_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS intelligence (
            id BLOB PRIMARY KEY,
            reporter_id BLOB NOT NULL,
            category INTEGER NOT NULL,
            danger_level INTEGER,
            region TEXT NOT NULL,
            summary TEXT NOT NULL,
            details BLOB,
            urgency INTEGER NOT NULL,
            reported_at INTEGER NOT NULL,
            expires_at INTEGER NOT NULL,
            hop_count INTEGER NOT NULL DEFAULT 0,
            verified BOOLEAN NOT NULL DEFAULT 0,
            confirmations INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (reporter_id) REFERENCES contacts(id)
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_intelligence_category
         ON intelligence(category, urgency)",
        [],
    )?;

    Ok(())
}

/// Messages table (E2E encrypted message history)
fn create_messages_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS messages (
            id BLOB PRIMARY KEY,
            contact_id BLOB NOT NULL,
            direction INTEGER NOT NULL,
            content BLOB NOT NULL,
            status INTEGER NOT NULL,
            created_at INTEGER NOT NULL,
            read_at INTEGER,
            expires_at INTEGER,
            FOREIGN KEY (contact_id) REFERENCES contacts(id)
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_messages_contact
         ON messages(contact_id, created_at)",
        [],
    )?;

    Ok(())
}

/// Trust relationships table (for web of trust)
fn create_trust_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS trust_relationships (
            truster_id BLOB NOT NULL,
            trustee_id BLOB NOT NULL,
            trust_level INTEGER NOT NULL,
            verification_method INTEGER NOT NULL,
            established_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            introduced_by BLOB,
            notes BLOB,
            PRIMARY KEY (truster_id, trustee_id),
            FOREIGN KEY (truster_id) REFERENCES contacts(id),
            FOREIGN KEY (trustee_id) REFERENCES contacts(id),
            FOREIGN KEY (introduced_by) REFERENCES contacts(id)
        )",
        [],
    )?;

    Ok(())
}

/// Settings table (key-value store for app configuration)
fn create_settings_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value BLOB NOT NULL
        )",
        [],
    )?;

    Ok(())
}

/// Metadata table (schema version, etc.)
fn create_metadata_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS metadata (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )?;

    Ok(())
}

/// Get current schema version
pub fn get_schema_version(conn: &Connection) -> Result<i32> {
    let version: String = conn
        .query_row(
            "SELECT value FROM metadata WHERE key = 'schema_version'",
            [],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| "0".to_string());

    Ok(version.parse().unwrap_or(0))
}

/// Set schema version
fn set_schema_version(conn: &Connection, version: i32) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO metadata (key, value) VALUES ('schema_version', ?)",
        [version.to_string()],
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_connection() -> (TempDir, Connection) {
        let tmp = TempDir::new().unwrap();
        let conn = Connection::open(tmp.path().join("test.db")).unwrap();
        (tmp, conn)
    }

    #[test]
    fn test_schema_creation() {
        let (_tmp, conn) = test_connection();

        init_schema(&conn).unwrap();

        // Verify all tables exist
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table'")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<rusqlite::Result<Vec<_>>>()
            .unwrap();

        assert!(tables.contains(&"identity".to_string()));
        assert!(tables.contains(&"contacts".to_string()));
        assert!(tables.contains(&"safe_houses".to_string()));
        assert!(tables.contains(&"transportation".to_string()));
        assert!(tables.contains(&"emergencies".to_string()));
        assert!(tables.contains(&"intelligence".to_string()));
        assert!(tables.contains(&"messages".to_string()));
        assert!(tables.contains(&"trust_relationships".to_string()));
        assert!(tables.contains(&"settings".to_string()));
        assert!(tables.contains(&"metadata".to_string()));
    }

    #[test]
    fn test_schema_version() {
        let (_tmp, conn) = test_connection();

        init_schema(&conn).unwrap();

        let version = get_schema_version(&conn).unwrap();
        assert_eq!(version, SCHEMA_VERSION);
    }

    #[test]
    fn test_foreign_keys_enabled() {
        let (_tmp, conn) = test_connection();

        init_schema(&conn).unwrap();

        let fk_enabled: i32 = conn
            .pragma_query_value(None, "foreign_keys", |row| row.get(0))
            .unwrap();

        assert_eq!(fk_enabled, 1);
    }
}
