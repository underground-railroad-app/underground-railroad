//! Identity repository - database operations for identities/personas

use crate::{
    identity::Identity,
    Result, PersonId, Fingerprint, CoarseTimestamp,
    storage::Database,
};
use rusqlite::{params, Row};

/// Repository for identity operations
pub struct IdentityRepository<'db> {
    db: &'db Database,
}

impl<'db> IdentityRepository<'db> {
    /// Create a new identity repository
    pub fn new(db: &'db Database) -> Self {
        Self { db }
    }

    /// Save an identity to the database
    pub fn save(&self, identity: &Identity) -> Result<()> {
        let conn = self.db.conn();

        // Serialize keypair (encrypted by database)
        let keypair_encrypted = bincode::serialize(&identity.keypair)
            .map_err(|e| crate::Error::Serialization(e.to_string()))?;

        // Serialize fingerprint
        let veilid_dht_key = vec![0u8; 32]; // Placeholder for Veilid DHT key
        let veilid_route = vec![0u8; 32]; // Placeholder for Veilid route

        conn.execute(
            "INSERT OR REPLACE INTO identity (
                id, name, fingerprint, veilid_dht_key, veilid_route,
                keypair_encrypted, created_at, is_primary
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                identity.id.0.as_bytes(),
                identity.name,
                identity.fingerprint.as_bytes(),
                veilid_dht_key,
                veilid_route,
                keypair_encrypted,
                identity.created_at.as_secs(),
                identity.is_primary,
            ],
        )?;

        Ok(())
    }

    /// Get an identity by ID
    pub fn get(&self, id: PersonId) -> Result<Option<Identity>> {
        let conn = self.db.conn();

        let result = conn.query_row(
            "SELECT id, name, fingerprint, keypair_encrypted, created_at, is_primary
             FROM identity WHERE id = ?1",
            params![id.0.as_bytes()],
            |row| self.row_to_identity(row),
        );

        match result {
            Ok(identity) => Ok(Some(identity)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Get the primary identity
    pub fn get_primary(&self) -> Result<Option<Identity>> {
        let conn = self.db.conn();

        let result = conn.query_row(
            "SELECT id, name, fingerprint, keypair_encrypted, created_at, is_primary
             FROM identity WHERE is_primary = 1 LIMIT 1",
            [],
            |row| self.row_to_identity(row),
        );

        match result {
            Ok(identity) => Ok(Some(identity)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// List all identities (personas)
    pub fn list(&self) -> Result<Vec<Identity>> {
        let conn = self.db.conn();

        let mut stmt = conn.prepare(
            "SELECT id, name, fingerprint, keypair_encrypted, created_at, is_primary
             FROM identity
             ORDER BY is_primary DESC, created_at ASC"
        )?;

        let identities = stmt.query_map([], |row| self.row_to_identity(row))?
            .map(|r| r.map_err(|e| e.into()))
            .collect::<Result<Vec<_>>>()?;

        Ok(identities)
    }

    /// Update identity name
    pub fn update_name(&self, id: PersonId, name: &str) -> Result<()> {
        let conn = self.db.conn();

        conn.execute(
            "UPDATE identity SET name = ?1 WHERE id = ?2",
            params![name, id.0.as_bytes()],
        )?;

        Ok(())
    }

    /// Set primary identity
    pub fn set_primary(&self, id: PersonId) -> Result<()> {
        let conn = self.db.conn();

        // Begin transaction
        let tx = conn.unchecked_transaction()?;

        // Unset all primary flags
        tx.execute("UPDATE identity SET is_primary = 0", [])?;

        // Set new primary
        tx.execute(
            "UPDATE identity SET is_primary = 1 WHERE id = ?1",
            params![id.0.as_bytes()],
        )?;

        tx.commit()?;

        Ok(())
    }

    /// Delete an identity
    pub fn delete(&self, id: PersonId) -> Result<()> {
        let conn = self.db.conn();

        // Check if this is the primary identity
        let is_primary: bool = conn.query_row(
            "SELECT is_primary FROM identity WHERE id = ?1",
            params![id.0.as_bytes()],
            |row| row.get(0),
        )?;

        if is_primary {
            return Err(crate::Error::PermissionDenied(
                "Cannot delete primary identity".to_string()
            ));
        }

        conn.execute(
            "DELETE FROM identity WHERE id = ?1",
            params![id.0.as_bytes()],
        )?;

        Ok(())
    }

    /// Count total identities
    pub fn count(&self) -> Result<usize> {
        let conn = self.db.conn();

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM identity",
            [],
            |row| row.get(0),
        )?;

        Ok(count as usize)
    }

    /// Check if an identity exists
    pub fn exists(&self, id: PersonId) -> Result<bool> {
        let conn = self.db.conn();

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM identity WHERE id = ?1",
            params![id.0.as_bytes()],
            |row| row.get(0),
        )?;

        Ok(count > 0)
    }

    /// Helper: Convert database row to Identity
    fn row_to_identity(&self, row: &Row) -> rusqlite::Result<Identity> {
        let id_bytes: Vec<u8> = row.get(0)?;
        let id = PersonId(uuid::Uuid::from_slice(&id_bytes).unwrap());

        let name: String = row.get(1)?;

        let fp_bytes: Vec<u8> = row.get(2)?;
        let mut fp_array = [0u8; 32];
        fp_array.copy_from_slice(&fp_bytes);
        let fingerprint = Fingerprint::new(fp_array);

        let keypair_encrypted: Vec<u8> = row.get(3)?;
        let keypair = bincode::deserialize(&keypair_encrypted)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        let created_at_secs: i64 = row.get(4)?;
        let created_at = CoarseTimestamp::from_datetime(
            chrono::DateTime::from_timestamp(created_at_secs, 0).unwrap()
        );

        let is_primary: bool = row.get(5)?;

        Ok(Identity {
            id,
            name,
            keypair,
            fingerprint,
            created_at,
            is_primary,
            veilid_mailbox: None, // Not stored in DB yet, managed via separate FFI calls
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{Database, DatabaseConfig};
    use tempfile::TempDir;
    use zeroize::Zeroizing;

    fn test_db() -> (TempDir, Database) {
        let tmp = TempDir::new().unwrap();
        let config = DatabaseConfig::new(tmp.path().join("test.db"));
        let key = Zeroizing::new([42u8; 32]);
        let db = Database::open(config, &key).unwrap();
        (tmp, db)
    }

    #[test]
    fn test_save_and_get() {
        let (_tmp, db) = test_db();
        let repo = IdentityRepository::new(&db);

        let identity = Identity::generate("Alice", true).unwrap();
        let id = identity.id;

        repo.save(&identity).unwrap();

        let retrieved = repo.get(id).unwrap();
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, identity.id);
        assert_eq!(retrieved.name, identity.name);
        assert_eq!(retrieved.is_primary, identity.is_primary);
    }

    #[test]
    fn test_get_primary() {
        let (_tmp, db) = test_db();
        let repo = IdentityRepository::new(&db);

        let primary = Identity::generate("Primary", true).unwrap();
        repo.save(&primary).unwrap();

        let secondary = Identity::generate("Secondary", false).unwrap();
        repo.save(&secondary).unwrap();

        let retrieved_primary = repo.get_primary().unwrap();
        assert!(retrieved_primary.is_some());
        assert_eq!(retrieved_primary.unwrap().id, primary.id);
    }

    #[test]
    fn test_list() {
        let (_tmp, db) = test_db();
        let repo = IdentityRepository::new(&db);

        let id1 = Identity::generate("Alice", true).unwrap();
        let id2 = Identity::generate("Bob", false).unwrap();
        let id3 = Identity::generate("Charlie", false).unwrap();

        repo.save(&id1).unwrap();
        repo.save(&id2).unwrap();
        repo.save(&id3).unwrap();

        let identities = repo.list().unwrap();
        assert_eq!(identities.len(), 3);

        // Primary should be first
        assert!(identities[0].is_primary);
    }

    #[test]
    fn test_update_name() {
        let (_tmp, db) = test_db();
        let repo = IdentityRepository::new(&db);

        let identity = Identity::generate("OldName", false).unwrap();
        let id = identity.id;
        repo.save(&identity).unwrap();

        repo.update_name(id, "NewName").unwrap();

        let updated = repo.get(id).unwrap().unwrap();
        assert_eq!(updated.name, "NewName");
    }

    #[test]
    fn test_set_primary() {
        let (_tmp, db) = test_db();
        let repo = IdentityRepository::new(&db);

        let id1 = Identity::generate("Alice", true).unwrap();
        let id2 = Identity::generate("Bob", false).unwrap();

        repo.save(&id1).unwrap();
        repo.save(&id2).unwrap();

        // Make Bob primary
        repo.set_primary(id2.id).unwrap();

        let primary = repo.get_primary().unwrap().unwrap();
        assert_eq!(primary.id, id2.id);

        // Alice should no longer be primary
        let alice = repo.get(id1.id).unwrap().unwrap();
        assert!(!alice.is_primary);
    }

    #[test]
    fn test_cannot_delete_primary() {
        let (_tmp, db) = test_db();
        let repo = IdentityRepository::new(&db);

        let primary = Identity::generate("Primary", true).unwrap();
        repo.save(&primary).unwrap();

        let result = repo.delete(primary.id);
        assert!(result.is_err());
    }

    #[test]
    fn test_can_delete_secondary() {
        let (_tmp, db) = test_db();
        let repo = IdentityRepository::new(&db);

        let primary = Identity::generate("Primary", true).unwrap();
        let secondary = Identity::generate("Secondary", false).unwrap();

        repo.save(&primary).unwrap();
        repo.save(&secondary).unwrap();

        // Can delete secondary
        repo.delete(secondary.id).unwrap();

        assert!(repo.get(secondary.id).unwrap().is_none());
        assert!(repo.get(primary.id).unwrap().is_some());
    }

    #[test]
    fn test_count() {
        let (_tmp, db) = test_db();
        let repo = IdentityRepository::new(&db);

        assert_eq!(repo.count().unwrap(), 0);

        repo.save(&Identity::generate("Alice", true).unwrap()).unwrap();
        assert_eq!(repo.count().unwrap(), 1);
    }

    #[test]
    fn test_exists() {
        let (_tmp, db) = test_db();
        let repo = IdentityRepository::new(&db);

        let identity = Identity::generate("Test", false).unwrap();
        let id = identity.id;

        assert!(!repo.exists(id).unwrap());

        repo.save(&identity).unwrap();
        assert!(repo.exists(id).unwrap());
    }
}
