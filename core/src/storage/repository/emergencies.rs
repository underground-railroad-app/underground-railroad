//! Emergency repository - database operations for emergencies

use crate::{
    assistance::{EmergencyRequest, EmergencyResponse, EmergencyStatus, EmergencyNeed},
    Result, PersonId, EmergencyId, Region, SecureBytes, CoarseTimestamp, Urgency,
    storage::Database,
};
use rusqlite::{params, Row};

/// Repository for emergency operations
pub struct EmergencyRepository<'db> {
    db: &'db Database,
}

impl<'db> EmergencyRepository<'db> {
    /// Create a new emergency repository
    pub fn new(db: &'db Database) -> Self {
        Self { db }
    }

    /// Save an emergency request to the database
    pub fn save(&self, emergency: &EmergencyRequest) -> Result<()> {
        let conn = self.db.conn();

        conn.execute(
            "INSERT OR REPLACE INTO emergencies (
                id, requester_id, needs, region, urgency,
                num_people, num_children, special_needs, notes,
                status, created_at, expires_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                emergency.id.0.as_bytes(),
                emergency.requester.as_ref().map(|id| id.0.as_bytes().to_vec()),
                serde_json::to_string(&emergency.needs).ok(),
                emergency.region.name,
                emergency.urgency as i32,
                emergency.num_people,
                emergency.num_children,
                emergency.special_needs.as_ref().map(|n| n.as_bytes()),
                emergency.notes.as_ref().map(|n| n.as_bytes()),
                emergency.status as i32,
                emergency.created_at.as_secs(),
                emergency.expires_at.as_secs(),
            ],
        )?;

        Ok(())
    }

    /// Get an emergency by ID
    pub fn get(&self, id: EmergencyId) -> Result<Option<EmergencyRequest>> {
        let conn = self.db.conn();

        let result = conn.query_row(
            "SELECT id, requester_id, needs, region, urgency,
                    num_people, num_children, special_needs, notes,
                    status, created_at, expires_at
             FROM emergencies WHERE id = ?1",
            params![id.0.as_bytes()],
            |row| self.row_to_emergency(row),
        );

        match result {
            Ok(emergency) => Ok(Some(emergency)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// List all emergencies
    pub fn list(&self) -> Result<Vec<EmergencyRequest>> {
        let conn = self.db.conn();

        let mut stmt = conn.prepare(
            "SELECT id, requester_id, needs, region, urgency,
                    num_people, num_children, special_needs, notes,
                    status, created_at, expires_at
             FROM emergencies
             ORDER BY urgency DESC, created_at DESC"
        )?;

        let emergencies = stmt.query_map([], |row| self.row_to_emergency(row))?
            .map(|r| r.map_err(|e| e.into()))
            .collect::<Result<Vec<_>>>()?;

        Ok(emergencies)
    }

    /// List active emergencies (not expired, not resolved)
    pub fn list_active(&self) -> Result<Vec<EmergencyRequest>> {
        let conn = self.db.conn();
        let now = CoarseTimestamp::now().as_secs();

        let mut stmt = conn.prepare(
            "SELECT id, requester_id, needs, region, urgency,
                    num_people, num_children, special_needs, notes,
                    status, created_at, expires_at
             FROM emergencies
             WHERE (status = ?1 OR status = ?2)
               AND expires_at > ?3
             ORDER BY urgency DESC, created_at DESC"
        )?;

        let emergencies = stmt.query_map(
            params![
                EmergencyStatus::Active as i32,
                EmergencyStatus::InProgress as i32,
                now
            ],
            |row| self.row_to_emergency(row)
        )?
        .map(|r| r.map_err(|e| e.into()))
        .collect::<Result<Vec<_>>>()?;

        Ok(emergencies)
    }

    /// List emergencies by urgency
    pub fn list_by_urgency(&self, min_urgency: Urgency) -> Result<Vec<EmergencyRequest>> {
        let conn = self.db.conn();
        let now = CoarseTimestamp::now().as_secs();

        let mut stmt = conn.prepare(
            "SELECT id, requester_id, needs, region, urgency,
                    num_people, num_children, special_needs, notes,
                    status, created_at, expires_at
             FROM emergencies
             WHERE urgency >= ?1
               AND (status = ?2 OR status = ?3)
               AND expires_at > ?4
             ORDER BY urgency DESC, created_at DESC"
        )?;

        let emergencies = stmt.query_map(
            params![
                min_urgency as i32,
                EmergencyStatus::Active as i32,
                EmergencyStatus::InProgress as i32,
                now
            ],
            |row| self.row_to_emergency(row)
        )?
        .map(|r| r.map_err(|e| e.into()))
        .collect::<Result<Vec<_>>>()?;

        Ok(emergencies)
    }

    /// List emergencies in a region
    pub fn list_by_region(&self, region: &str) -> Result<Vec<EmergencyRequest>> {
        let conn = self.db.conn();
        let now = CoarseTimestamp::now().as_secs();

        let mut stmt = conn.prepare(
            "SELECT id, requester_id, needs, region, urgency,
                    num_people, num_children, special_needs, notes,
                    status, created_at, expires_at
             FROM emergencies
             WHERE region LIKE ?1
               AND (status = ?2 OR status = ?3)
               AND expires_at > ?4
             ORDER BY urgency DESC, created_at DESC"
        )?;

        let region_pattern = format!("%{}%", region);
        let emergencies = stmt.query_map(
            params![
                region_pattern,
                EmergencyStatus::Active as i32,
                EmergencyStatus::InProgress as i32,
                now
            ],
            |row| self.row_to_emergency(row)
        )?
        .map(|r| r.map_err(|e| e.into()))
        .collect::<Result<Vec<_>>>()?;

        Ok(emergencies)
    }

    /// Update emergency status
    pub fn update_status(&self, id: EmergencyId, status: EmergencyStatus) -> Result<()> {
        let conn = self.db.conn();

        conn.execute(
            "UPDATE emergencies SET status = ?1 WHERE id = ?2",
            params![status as i32, id.0.as_bytes()],
        )?;

        Ok(())
    }

    /// Mark emergency as resolved
    pub fn resolve(&self, id: EmergencyId) -> Result<()> {
        self.update_status(id, EmergencyStatus::Resolved)
    }

    /// Mark emergency as in progress
    pub fn start_helping(&self, id: EmergencyId) -> Result<()> {
        self.update_status(id, EmergencyStatus::InProgress)
    }

    /// Delete an emergency
    pub fn delete(&self, id: EmergencyId) -> Result<()> {
        let conn = self.db.conn();

        conn.execute(
            "DELETE FROM emergencies WHERE id = ?1",
            params![id.0.as_bytes()],
        )?;

        Ok(())
    }

    /// Count total emergencies
    pub fn count(&self) -> Result<usize> {
        let conn = self.db.conn();

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM emergencies",
            [],
            |row| row.get(0),
        )?;

        Ok(count as usize)
    }

    /// Count active emergencies
    pub fn count_active(&self) -> Result<usize> {
        let conn = self.db.conn();
        let now = CoarseTimestamp::now().as_secs();

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM emergencies
             WHERE (status = ?1 OR status = ?2)
               AND expires_at > ?3",
            params![
                EmergencyStatus::Active as i32,
                EmergencyStatus::InProgress as i32,
                now
            ],
            |row| row.get(0),
        )?;

        Ok(count as usize)
    }

    /// Count critical emergencies
    pub fn count_critical(&self) -> Result<usize> {
        let conn = self.db.conn();
        let now = CoarseTimestamp::now().as_secs();

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM emergencies
             WHERE urgency = ?1
               AND (status = ?2 OR status = ?3)
               AND expires_at > ?4",
            params![
                Urgency::Critical as i32,
                EmergencyStatus::Active as i32,
                EmergencyStatus::InProgress as i32,
                now
            ],
            |row| row.get(0),
        )?;

        Ok(count as usize)
    }

    /// Cleanup expired emergencies
    pub fn cleanup_expired(&self) -> Result<usize> {
        let conn = self.db.conn();
        let now = CoarseTimestamp::now().as_secs();

        let deleted = conn.execute(
            "UPDATE emergencies
             SET status = ?1
             WHERE expires_at <= ?2
               AND status != ?3",
            params![
                EmergencyStatus::Expired as i32,
                now,
                EmergencyStatus::Resolved as i32
            ],
        )?;

        Ok(deleted)
    }

    /// Helper: Convert database row to EmergencyRequest
    fn row_to_emergency(&self, row: &Row) -> rusqlite::Result<EmergencyRequest> {
        let id_bytes: Vec<u8> = row.get(0)?;
        let id = EmergencyId(uuid::Uuid::from_slice(&id_bytes).unwrap());

        let requester_bytes: Option<Vec<u8>> = row.get(1)?;
        let requester = requester_bytes.map(|bytes| {
            PersonId(uuid::Uuid::from_slice(&bytes).unwrap())
        });

        let needs_json: Option<String> = row.get(2)?;
        let needs: Vec<EmergencyNeed> = needs_json
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_default();

        let region_name: String = row.get(3)?;
        let region = Region::new(region_name);

        let urgency_int: i32 = row.get(4)?;
        let urgency = match urgency_int {
            1 => Urgency::Low,
            2 => Urgency::Medium,
            3 => Urgency::High,
            4 => Urgency::Critical,
            _ => Urgency::Medium,
        };

        let num_people: u32 = row.get(5)?;
        let num_children: u32 = row.get(6)?;

        let special_needs_bytes: Option<Vec<u8>> = row.get(7)?;
        let special_needs = special_needs_bytes.map(SecureBytes::new);

        let notes_bytes: Option<Vec<u8>> = row.get(8)?;
        let notes = notes_bytes.map(SecureBytes::new);

        let status_int: i32 = row.get(9)?;
        let status = match status_int {
            0 => EmergencyStatus::Active,
            1 => EmergencyStatus::InProgress,
            2 => EmergencyStatus::Resolved,
            3 => EmergencyStatus::Expired,
            4 => EmergencyStatus::Cancelled,
            _ => EmergencyStatus::Active,
        };

        let created_at_secs: i64 = row.get(10)?;
        let created_at = CoarseTimestamp::from_datetime(
            chrono::DateTime::from_timestamp(created_at_secs, 0).unwrap()
        );

        let expires_at_secs: i64 = row.get(11)?;
        let expires_at = CoarseTimestamp::from_datetime(
            chrono::DateTime::from_timestamp(expires_at_secs, 0).unwrap()
        );

        Ok(EmergencyRequest {
            id,
            requester,
            needs,
            region,
            urgency,
            num_people,
            num_children,
            special_needs,
            notes,
            status,
            created_at,
            expires_at,
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

    fn test_emergency() -> EmergencyRequest {
        EmergencyRequest::new(
            Some(PersonId::new()),
            vec![EmergencyNeed::SafeShelter],
            Region::new("Test Region"),
            Urgency::High,
            2,
        )
    }

    #[test]
    fn test_save_and_get_emergency() {
        let (_tmp, db) = test_db();
        let repo = EmergencyRepository::new(&db);

        let emergency = test_emergency();
        let id = emergency.id;

        // Save
        repo.save(&emergency).unwrap();

        // Get
        let retrieved = repo.get(id).unwrap();
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, emergency.id);
        assert_eq!(retrieved.urgency, emergency.urgency);
        assert_eq!(retrieved.num_people, emergency.num_people);
    }

    #[test]
    fn test_list_active() {
        let (_tmp, db) = test_db();
        let repo = EmergencyRepository::new(&db);

        // Create active emergency
        let active = test_emergency();
        repo.save(&active).unwrap();

        // Create resolved emergency
        let mut resolved = test_emergency();
        resolved.status = EmergencyStatus::Resolved;
        repo.save(&resolved).unwrap();

        // List active should only return the first one
        let active_list = repo.list_active().unwrap();
        assert_eq!(active_list.len(), 1);
        assert_eq!(active_list[0].id, active.id);
    }

    #[test]
    fn test_list_by_urgency() {
        let (_tmp, db) = test_db();
        let repo = EmergencyRepository::new(&db);

        // Create critical emergency
        let mut critical = test_emergency();
        critical.urgency = Urgency::Critical;
        repo.save(&critical).unwrap();

        // Create low emergency
        let mut low = test_emergency();
        low.urgency = Urgency::Low;
        repo.save(&low).unwrap();

        // List high+ urgency
        let high_urgency = repo.list_by_urgency(Urgency::High).unwrap();
        assert_eq!(high_urgency.len(), 1);
        assert_eq!(high_urgency[0].id, critical.id);
    }

    #[test]
    fn test_update_status() {
        let (_tmp, db) = test_db();
        let repo = EmergencyRepository::new(&db);

        let emergency = test_emergency();
        let id = emergency.id;
        repo.save(&emergency).unwrap();

        // Update status
        repo.update_status(id, EmergencyStatus::InProgress).unwrap();

        // Verify
        let updated = repo.get(id).unwrap().unwrap();
        assert_eq!(updated.status, EmergencyStatus::InProgress);
    }

    #[test]
    fn test_resolve() {
        let (_tmp, db) = test_db();
        let repo = EmergencyRepository::new(&db);

        let emergency = test_emergency();
        let id = emergency.id;
        repo.save(&emergency).unwrap();

        // Resolve
        repo.resolve(id).unwrap();

        // Verify
        let resolved = repo.get(id).unwrap().unwrap();
        assert_eq!(resolved.status, EmergencyStatus::Resolved);

        // Should not appear in active list
        let active = repo.list_active().unwrap();
        assert_eq!(active.len(), 0);
    }

    #[test]
    fn test_count() {
        let (_tmp, db) = test_db();
        let repo = EmergencyRepository::new(&db);

        assert_eq!(repo.count().unwrap(), 0);
        assert_eq!(repo.count_active().unwrap(), 0);

        repo.save(&test_emergency()).unwrap();
        assert_eq!(repo.count().unwrap(), 1);
        assert_eq!(repo.count_active().unwrap(), 1);
    }

    #[test]
    fn test_count_critical() {
        let (_tmp, db) = test_db();
        let repo = EmergencyRepository::new(&db);

        let mut critical = test_emergency();
        critical.urgency = Urgency::Critical;
        repo.save(&critical).unwrap();

        let low = test_emergency();
        repo.save(&low).unwrap();

        assert_eq!(repo.count_critical().unwrap(), 1);
    }

    #[test]
    fn test_delete() {
        let (_tmp, db) = test_db();
        let repo = EmergencyRepository::new(&db);

        let emergency = test_emergency();
        let id = emergency.id;
        repo.save(&emergency).unwrap();

        // Delete
        repo.delete(id).unwrap();

        // Verify
        assert!(repo.get(id).unwrap().is_none());
    }
}
