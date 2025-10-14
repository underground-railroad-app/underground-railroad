//! Safe house repository - database operations for safe houses

use crate::{
    assistance::{SafeHouse, SafeHouseCapability, SafeHouseStatus, Accommodation},
    Result, PersonId, SafeHouseId, Region, SecureBytes, CoarseTimestamp,
    storage::Database,
};
use rusqlite::{params, Row};

/// Repository for safe house operations
pub struct SafeHouseRepository<'db> {
    db: &'db Database,
}

impl<'db> SafeHouseRepository<'db> {
    /// Create a new safe house repository
    pub fn new(db: &'db Database) -> Self {
        Self { db }
    }

    /// Save a safe house to the database
    pub fn save(&self, safe_house: &SafeHouse) -> Result<()> {
        let conn = self.db.conn();

        conn.execute(
            "INSERT OR REPLACE INTO safe_houses (
                id, operator_id, name, region, capabilities,
                capacity, current_occupancy, status, accommodations,
                max_stay_days, notes, verified, registered_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                safe_house.id.0.as_bytes(),
                safe_house.operator.0.as_bytes(),
                safe_house.name,
                safe_house.region.name,
                serde_json::to_string(&safe_house.capabilities).ok(),
                safe_house.capacity,
                safe_house.current_occupancy,
                safe_house.status as i32,
                serde_json::to_string(&safe_house.accommodations).ok(),
                safe_house.max_stay_days,
                safe_house.notes.as_ref().map(|n| n.as_bytes()),
                safe_house.verified,
                safe_house.registered_at.as_secs(),
                safe_house.updated_at.as_secs(),
            ],
        )?;

        Ok(())
    }

    /// Get a safe house by ID
    pub fn get(&self, id: SafeHouseId) -> Result<Option<SafeHouse>> {
        let conn = self.db.conn();

        let result = conn.query_row(
            "SELECT id, operator_id, name, region, capabilities,
                    capacity, current_occupancy, status, accommodations,
                    max_stay_days, notes, verified, registered_at, updated_at
             FROM safe_houses WHERE id = ?1",
            params![id.0.as_bytes()],
            |row| self.row_to_safe_house(row),
        );

        match result {
            Ok(safe_house) => Ok(Some(safe_house)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// List all safe houses
    pub fn list(&self) -> Result<Vec<SafeHouse>> {
        let conn = self.db.conn();

        let mut stmt = conn.prepare(
            "SELECT id, operator_id, name, region, capabilities,
                    capacity, current_occupancy, status, accommodations,
                    max_stay_days, notes, verified, registered_at, updated_at
             FROM safe_houses
             ORDER BY name"
        )?;

        let safe_houses = stmt.query_map([], |row| self.row_to_safe_house(row))?
            .map(|r| r.map_err(|e| e.into()))
            .collect::<Result<Vec<_>>>()?;

        Ok(safe_houses)
    }

    /// List available safe houses (with capacity)
    pub fn list_available(&self) -> Result<Vec<SafeHouse>> {
        let conn = self.db.conn();

        let mut stmt = conn.prepare(
            "SELECT id, operator_id, name, region, capabilities,
                    capacity, current_occupancy, status, accommodations,
                    max_stay_days, notes, verified, registered_at, updated_at
             FROM safe_houses
             WHERE (status = ?1 OR status = ?2)
               AND current_occupancy < capacity
             ORDER BY verified DESC, current_occupancy ASC"
        )?;

        let safe_houses = stmt.query_map(
            params![
                SafeHouseStatus::Available as i32,
                SafeHouseStatus::AvailableWithNotice as i32
            ],
            |row| self.row_to_safe_house(row)
        )?
        .map(|r| r.map_err(|e| e.into()))
        .collect::<Result<Vec<_>>>()?;

        Ok(safe_houses)
    }

    /// List safe houses by capability
    pub fn list_by_capability(&self, capability: SafeHouseCapability) -> Result<Vec<SafeHouse>> {
        let conn = self.db.conn();

        let mut stmt = conn.prepare(
            "SELECT id, operator_id, name, region, capabilities,
                    capacity, current_occupancy, status, accommodations,
                    max_stay_days, notes, verified, registered_at, updated_at
             FROM safe_houses
             WHERE capabilities LIKE ?1
             ORDER BY verified DESC, name"
        )?;

        let capability_str = format!("%{:?}%", capability);
        let safe_houses = stmt.query_map([capability_str], |row| self.row_to_safe_house(row))?
            .map(|r| r.map_err(|e| e.into()))
            .collect::<Result<Vec<_>>>()?;

        Ok(safe_houses)
    }

    /// List safe houses in a region
    pub fn list_by_region(&self, region: &str) -> Result<Vec<SafeHouse>> {
        let conn = self.db.conn();

        let mut stmt = conn.prepare(
            "SELECT id, operator_id, name, region, capabilities,
                    capacity, current_occupancy, status, accommodations,
                    max_stay_days, notes, verified, registered_at, updated_at
             FROM safe_houses
             WHERE region LIKE ?1
             ORDER BY verified DESC, name"
        )?;

        let region_pattern = format!("%{}%", region);
        let safe_houses = stmt.query_map([region_pattern], |row| self.row_to_safe_house(row))?
            .map(|r| r.map_err(|e| e.into()))
            .collect::<Result<Vec<_>>>()?;

        Ok(safe_houses)
    }

    /// Update safe house occupancy
    pub fn update_occupancy(&self, id: SafeHouseId, occupancy: u32) -> Result<()> {
        let conn = self.db.conn();

        let now = CoarseTimestamp::now();
        conn.execute(
            "UPDATE safe_houses
             SET current_occupancy = ?1, updated_at = ?2
             WHERE id = ?3",
            params![occupancy, now.as_secs(), id.0.as_bytes()],
        )?;

        Ok(())
    }

    /// Update safe house status
    pub fn update_status(&self, id: SafeHouseId, status: SafeHouseStatus) -> Result<()> {
        let conn = self.db.conn();

        let now = CoarseTimestamp::now();
        conn.execute(
            "UPDATE safe_houses
             SET status = ?1, updated_at = ?2
             WHERE id = ?3",
            params![status as i32, now.as_secs(), id.0.as_bytes()],
        )?;

        Ok(())
    }

    /// Mark safe house as verified
    pub fn verify(&self, id: SafeHouseId) -> Result<()> {
        let conn = self.db.conn();

        let now = CoarseTimestamp::now();
        conn.execute(
            "UPDATE safe_houses
             SET verified = 1, updated_at = ?1
             WHERE id = ?2",
            params![now.as_secs(), id.0.as_bytes()],
        )?;

        Ok(())
    }

    /// Delete a safe house
    pub fn delete(&self, id: SafeHouseId) -> Result<()> {
        let conn = self.db.conn();

        conn.execute(
            "DELETE FROM safe_houses WHERE id = ?1",
            params![id.0.as_bytes()],
        )?;

        Ok(())
    }

    /// Count total safe houses
    pub fn count(&self) -> Result<usize> {
        let conn = self.db.conn();

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM safe_houses",
            [],
            |row| row.get(0),
        )?;

        Ok(count as usize)
    }

    /// Count available safe houses
    pub fn count_available(&self) -> Result<usize> {
        let conn = self.db.conn();

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM safe_houses
             WHERE (status = ?1 OR status = ?2)
               AND current_occupancy < capacity",
            params![
                SafeHouseStatus::Available as i32,
                SafeHouseStatus::AvailableWithNotice as i32
            ],
            |row| row.get(0),
        )?;

        Ok(count as usize)
    }

    /// Helper: Convert database row to SafeHouse
    fn row_to_safe_house(&self, row: &Row) -> rusqlite::Result<SafeHouse> {
        let id_bytes: Vec<u8> = row.get(0)?;
        let id = SafeHouseId(uuid::Uuid::from_slice(&id_bytes).unwrap());

        let operator_bytes: Vec<u8> = row.get(1)?;
        let operator = PersonId(uuid::Uuid::from_slice(&operator_bytes).unwrap());

        let name: String = row.get(2)?;

        let region_name: String = row.get(3)?;
        let region = Region::new(region_name);

        let capabilities_json: Option<String> = row.get(4)?;
        let capabilities = capabilities_json
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_default();

        let capacity: u32 = row.get(5)?;
        let current_occupancy: u32 = row.get(6)?;

        let status_int: i32 = row.get(7)?;
        let status = match status_int {
            0 => SafeHouseStatus::Available,
            1 => SafeHouseStatus::AvailableWithNotice,
            2 => SafeHouseStatus::Occupied,
            3 => SafeHouseStatus::TemporarilyUnavailable,
            4 => SafeHouseStatus::Closed,
            _ => SafeHouseStatus::TemporarilyUnavailable,
        };

        let accommodations_json: Option<String> = row.get(8)?;
        let accommodations = accommodations_json
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_default();

        let max_stay_days: Option<u32> = row.get(9)?;

        let notes_bytes: Option<Vec<u8>> = row.get(10)?;
        let notes = notes_bytes.map(SecureBytes::new);

        let verified: bool = row.get(11)?;

        let registered_at_secs: i64 = row.get(12)?;
        let registered_at = CoarseTimestamp::from_datetime(
            chrono::DateTime::from_timestamp(registered_at_secs, 0).unwrap()
        );

        let updated_at_secs: i64 = row.get(13)?;
        let updated_at = CoarseTimestamp::from_datetime(
            chrono::DateTime::from_timestamp(updated_at_secs, 0).unwrap()
        );

        Ok(SafeHouse {
            id,
            operator,
            name,
            region,
            capabilities,
            capacity,
            current_occupancy,
            status,
            accommodations,
            max_stay_days,
            notes,
            verified,
            registered_at,
            updated_at,
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

    fn test_safe_house() -> SafeHouse {
        SafeHouse::new(
            PersonId::new(),
            "Test Safe House",
            Region::new("Test Region"),
            4,
        )
    }

    #[test]
    fn test_save_and_get_safe_house() {
        let (_tmp, db) = test_db();
        let repo = SafeHouseRepository::new(&db);

        let mut house = test_safe_house();
        house.add_capability(SafeHouseCapability::Shelter);
        house.add_capability(SafeHouseCapability::Food);

        let id = house.id;

        // Save
        repo.save(&house).unwrap();

        // Get
        let retrieved = repo.get(id).unwrap();
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, house.id);
        assert_eq!(retrieved.name, house.name);
        assert_eq!(retrieved.capacity, house.capacity);
    }

    #[test]
    fn test_list_available() {
        let (_tmp, db) = test_db();
        let repo = SafeHouseRepository::new(&db);

        // Create available house
        let mut available = test_safe_house();
        available.status = SafeHouseStatus::Available;
        available.current_occupancy = 1;
        repo.save(&available).unwrap();

        // Create occupied house
        let mut occupied = test_safe_house();
        occupied.status = SafeHouseStatus::Occupied;
        occupied.current_occupancy = occupied.capacity;
        repo.save(&occupied).unwrap();

        // List available should only return the first one
        let available_list = repo.list_available().unwrap();
        assert_eq!(available_list.len(), 1);
        assert_eq!(available_list[0].id, available.id);
    }

    #[test]
    fn test_update_occupancy() {
        let (_tmp, db) = test_db();
        let repo = SafeHouseRepository::new(&db);

        let house = test_safe_house();
        let id = house.id;
        repo.save(&house).unwrap();

        // Update occupancy
        repo.update_occupancy(id, 3).unwrap();

        // Verify
        let updated = repo.get(id).unwrap().unwrap();
        assert_eq!(updated.current_occupancy, 3);
    }

    #[test]
    fn test_verify_safe_house() {
        let (_tmp, db) = test_db();
        let repo = SafeHouseRepository::new(&db);

        let house = test_safe_house();
        let id = house.id;
        assert!(!house.verified);

        repo.save(&house).unwrap();

        // Verify
        repo.verify(id).unwrap();

        // Check
        let verified = repo.get(id).unwrap().unwrap();
        assert!(verified.verified);
    }

    #[test]
    fn test_count() {
        let (_tmp, db) = test_db();
        let repo = SafeHouseRepository::new(&db);

        assert_eq!(repo.count().unwrap(), 0);

        repo.save(&test_safe_house()).unwrap();
        assert_eq!(repo.count().unwrap(), 1);

        repo.save(&test_safe_house()).unwrap();
        assert_eq!(repo.count().unwrap(), 2);
    }

    #[test]
    fn test_delete() {
        let (_tmp, db) = test_db();
        let repo = SafeHouseRepository::new(&db);

        let house = test_safe_house();
        let id = house.id;
        repo.save(&house).unwrap();

        // Delete
        repo.delete(id).unwrap();

        // Verify
        assert!(repo.get(id).unwrap().is_none());
    }
}
