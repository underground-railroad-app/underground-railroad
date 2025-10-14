//! Contact repository - database operations for contacts

use crate::{
    trust::{Contact, ContactInfo, Capability},
    Result, PersonId, Fingerprint, SecureBytes, TrustLevel, CoarseTimestamp,
    storage::Database,
};
use rusqlite::{params, Row};

/// Repository for contact operations
pub struct ContactRepository<'db> {
    db: &'db Database,
}

impl<'db> ContactRepository<'db> {
    /// Create a new contact repository
    pub fn new(db: &'db Database) -> Self {
        Self { db }
    }

    /// Save a contact to the database
    pub fn save(&self, contact: &Contact) -> Result<()> {
        let conn = self.db.conn();

        conn.execute(
            "INSERT OR REPLACE INTO contacts (
                id, name, fingerprint, veilid_route, trust_level,
                verification_method, region, languages, capabilities,
                tags, added_at, last_contact, introduced_by, notes, available
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            params![
                contact.id.0.as_bytes(),
                contact.info.name,
                contact.fingerprint.as_bytes(),
                contact.veilid_route.as_bytes(),
                contact.trust_level as i32,
                0, // verification_method placeholder
                contact.info.region.as_ref().map(|r| &r.name),
                serde_json::to_string(&contact.info.languages).ok(),
                serde_json::to_string(&contact.info.capabilities).ok(),
                serde_json::to_string(&contact.tags).ok(),
                contact.added_at.as_secs(),
                contact.last_contact.map(|t| t.as_secs()),
                contact.introduced_by.as_ref().map(|id| id.0.as_bytes().to_vec()),
                contact.notes.as_ref().map(|n| n.as_bytes()),
                contact.available,
            ],
        )?;

        Ok(())
    }

    /// Get a contact by ID
    pub fn get(&self, id: PersonId) -> Result<Option<Contact>> {
        let conn = self.db.conn();

        let result = conn.query_row(
            "SELECT id, name, fingerprint, veilid_route, trust_level,
                    region, languages, capabilities, tags, added_at,
                    last_contact, introduced_by, notes, available
             FROM contacts WHERE id = ?1",
            params![id.0.as_bytes()],
            |row| self.row_to_contact(row),
        );

        match result {
            Ok(contact) => Ok(Some(contact)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// List all contacts
    pub fn list(&self) -> Result<Vec<Contact>> {
        let conn = self.db.conn();

        let mut stmt = conn.prepare(
            "SELECT id, name, fingerprint, veilid_route, trust_level,
                    region, languages, capabilities, tags, added_at,
                    last_contact, introduced_by, notes, available
             FROM contacts
             ORDER BY name"
        )?;

        let contacts = stmt.query_map([], |row| self.row_to_contact(row))?
            .map(|r| r.map_err(|e| e.into()))
            .collect::<Result<Vec<_>>>()?;

        Ok(contacts)
    }

    /// List contacts by trust level
    pub fn list_by_trust(&self, min_level: TrustLevel) -> Result<Vec<Contact>> {
        let conn = self.db.conn();

        let mut stmt = conn.prepare(
            "SELECT id, name, fingerprint, veilid_route, trust_level,
                    region, languages, capabilities, tags, added_at,
                    last_contact, introduced_by, notes, available
             FROM contacts
             WHERE trust_level >= ?1
             ORDER BY trust_level DESC, name"
        )?;

        let contacts = stmt.query_map([min_level as i32], |row| self.row_to_contact(row))?
            .map(|r| r.map_err(|e| e.into()))
            .collect::<Result<Vec<_>>>()?;

        Ok(contacts)
    }

    /// List contacts with a specific capability
    pub fn list_by_capability(&self, capability: Capability) -> Result<Vec<Contact>> {
        let conn = self.db.conn();

        let mut stmt = conn.prepare(
            "SELECT id, name, fingerprint, veilid_route, trust_level,
                    region, languages, capabilities, tags, added_at,
                    last_contact, introduced_by, notes, available
             FROM contacts
             WHERE capabilities LIKE ?1
             ORDER BY name"
        )?;

        let capability_str = format!("%{:?}%", capability);
        let contacts = stmt.query_map([capability_str], |row| self.row_to_contact(row))?
            .map(|r| r.map_err(|e| e.into()))
            .collect::<Result<Vec<_>>>()?;

        Ok(contacts)
    }

    /// Update contact trust level
    pub fn update_trust(&self, id: PersonId, trust_level: TrustLevel) -> Result<()> {
        let conn = self.db.conn();

        conn.execute(
            "UPDATE contacts SET trust_level = ?1 WHERE id = ?2",
            params![trust_level as i32, id.0.as_bytes()],
        )?;

        Ok(())
    }

    /// Update last contact time
    pub fn update_last_contact(&self, id: PersonId) -> Result<()> {
        let conn = self.db.conn();

        let now = CoarseTimestamp::now();
        conn.execute(
            "UPDATE contacts SET last_contact = ?1 WHERE id = ?2",
            params![now.as_secs(), id.0.as_bytes()],
        )?;

        Ok(())
    }

    /// Delete a contact
    pub fn delete(&self, id: PersonId) -> Result<()> {
        let conn = self.db.conn();

        conn.execute(
            "DELETE FROM contacts WHERE id = ?1",
            params![id.0.as_bytes()],
        )?;

        Ok(())
    }

    /// Count total contacts
    pub fn count(&self) -> Result<usize> {
        let conn = self.db.conn();

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM contacts",
            [],
            |row| row.get(0),
        )?;

        Ok(count as usize)
    }

    /// Search contacts by name
    pub fn search(&self, query: &str) -> Result<Vec<Contact>> {
        let conn = self.db.conn();

        let mut stmt = conn.prepare(
            "SELECT id, name, fingerprint, veilid_route, trust_level,
                    region, languages, capabilities, tags, added_at,
                    last_contact, introduced_by, notes, available
             FROM contacts
             WHERE name LIKE ?1
             ORDER BY name"
        )?;

        let search_pattern = format!("%{}%", query);
        let contacts = stmt.query_map([search_pattern], |row| self.row_to_contact(row))?
            .map(|r| r.map_err(|e| e.into()))
            .collect::<Result<Vec<_>>>()?;

        Ok(contacts)
    }

    /// Helper: Convert database row to Contact
    fn row_to_contact(&self, row: &Row) -> rusqlite::Result<Contact> {
        let id_bytes: Vec<u8> = row.get(0)?;
        let id = PersonId(uuid::Uuid::from_slice(&id_bytes).unwrap());

        let name: String = row.get(1)?;

        let fp_bytes: Vec<u8> = row.get(2)?;
        let mut fp_array = [0u8; 32];
        fp_array.copy_from_slice(&fp_bytes);
        let fingerprint = Fingerprint::new(fp_array);

        let route_bytes: Vec<u8> = row.get(3)?;
        let veilid_route = SecureBytes::new(route_bytes);

        let trust_level_int: i32 = row.get(4)?;
        let trust_level = match trust_level_int {
            0 => TrustLevel::Blocked,
            1 => TrustLevel::Unknown,
            2 => TrustLevel::Introduced,
            3 => TrustLevel::VerifiedRemote,
            4 => TrustLevel::VerifiedInPerson,
            _ => TrustLevel::Unknown,
        };

        let region_name: Option<String> = row.get(5)?;
        let region = region_name.map(crate::Region::new);

        let languages_json: Option<String> = row.get(6)?;
        let languages = languages_json
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_default();

        let capabilities_json: Option<String> = row.get(7)?;
        let capabilities = capabilities_json
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_default();

        let tags_json: Option<String> = row.get(8)?;
        let tags = tags_json
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_default();

        let added_at_secs: i64 = row.get(9)?;
        let added_at = CoarseTimestamp::from_datetime(
            chrono::DateTime::from_timestamp(added_at_secs, 0).unwrap()
        );

        let last_contact_secs: Option<i64> = row.get(10)?;
        let last_contact = last_contact_secs.map(|secs| {
            CoarseTimestamp::from_datetime(
                chrono::DateTime::from_timestamp(secs, 0).unwrap()
            )
        });

        let introduced_by_bytes: Option<Vec<u8>> = row.get(11)?;
        let introduced_by = introduced_by_bytes.map(|bytes| {
            PersonId(uuid::Uuid::from_slice(&bytes).unwrap())
        });

        let notes_bytes: Option<Vec<u8>> = row.get(12)?;
        let notes = notes_bytes.map(SecureBytes::new);

        let available: bool = row.get(13)?;

        Ok(Contact {
            id,
            info: ContactInfo {
                name,
                region,
                avatar_hash: None,
                languages,
                capabilities,
            },
            trust_level,
            fingerprint,
            veilid_route,
            added_at,
            last_contact,
            introduced_by,
            notes,
            available,
            tags,
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

    fn test_contact() -> Contact {
        Contact::new(
            PersonId::new(),
            "Test Contact",
            Fingerprint::new([0u8; 32]),
            SecureBytes::new(vec![1, 2, 3, 4]),
            TrustLevel::VerifiedRemote,
        )
    }

    #[test]
    fn test_save_and_get_contact() {
        let (_tmp, db) = test_db();
        let repo = ContactRepository::new(&db);

        let contact = test_contact();
        let id = contact.id;

        // Save
        repo.save(&contact).unwrap();

        // Get
        let retrieved = repo.get(id).unwrap();
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, contact.id);
        assert_eq!(retrieved.info.name, contact.info.name);
        assert_eq!(retrieved.trust_level, contact.trust_level);
    }

    #[test]
    fn test_list_contacts() {
        let (_tmp, db) = test_db();
        let repo = ContactRepository::new(&db);

        // Save multiple contacts
        for i in 0..5 {
            let mut contact = test_contact();
            contact.info.name = format!("Contact {}", i);
            repo.save(&contact).unwrap();
        }

        let contacts = repo.list().unwrap();
        assert_eq!(contacts.len(), 5);
    }

    #[test]
    fn test_update_trust() {
        let (_tmp, db) = test_db();
        let repo = ContactRepository::new(&db);

        let contact = test_contact();
        let id = contact.id;
        repo.save(&contact).unwrap();

        // Update trust
        repo.update_trust(id, TrustLevel::VerifiedInPerson).unwrap();

        // Verify
        let updated = repo.get(id).unwrap().unwrap();
        assert_eq!(updated.trust_level, TrustLevel::VerifiedInPerson);
    }

    #[test]
    fn test_delete_contact() {
        let (_tmp, db) = test_db();
        let repo = ContactRepository::new(&db);

        let contact = test_contact();
        let id = contact.id;
        repo.save(&contact).unwrap();

        // Delete
        repo.delete(id).unwrap();

        // Verify
        assert!(repo.get(id).unwrap().is_none());
    }

    #[test]
    fn test_search_contacts() {
        let (_tmp, db) = test_db();
        let repo = ContactRepository::new(&db);

        let mut contact = test_contact();
        contact.info.name = "Alice Smith".to_string();
        repo.save(&contact).unwrap();

        let results = repo.search("Alice").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].info.name, "Alice Smith");
    }

    #[test]
    fn test_count_contacts() {
        let (_tmp, db) = test_db();
        let repo = ContactRepository::new(&db);

        assert_eq!(repo.count().unwrap(), 0);

        repo.save(&test_contact()).unwrap();
        assert_eq!(repo.count().unwrap(), 1);

        repo.save(&test_contact()).unwrap();
        assert_eq!(repo.count().unwrap(), 2);
    }
}
