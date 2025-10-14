//! Intelligence repository - database operations for intelligence reports

use crate::{
    assistance::{IntelligenceReport, IntelligenceCategory, DangerLevel},
    Result, PersonId, Region, SecureBytes, CoarseTimestamp, Urgency,
    storage::Database,
};
use rusqlite::{params, Row};
use uuid::Uuid;

/// Repository for intelligence operations
pub struct IntelligenceRepository<'db> {
    db: &'db Database,
}

impl<'db> IntelligenceRepository<'db> {
    /// Create a new intelligence repository
    pub fn new(db: &'db Database) -> Self {
        Self { db }
    }

    /// Save an intelligence report to the database
    pub fn save(&self, report: &IntelligenceReport) -> Result<()> {
        let conn = self.db.conn();

        conn.execute(
            "INSERT OR REPLACE INTO intelligence (
                id, reporter_id, category, danger_level, region,
                summary, details, urgency, reported_at, expires_at,
                hop_count, verified, confirmations
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                report.id.as_bytes(),
                report.reporter.0.as_bytes(),
                report.category as i32,
                report.danger_level.map(|d| d as i32),
                report.region.name,
                report.summary,
                report.details.as_ref().map(|d| d.as_bytes()),
                report.urgency as i32,
                report.reported_at.as_secs(),
                report.expires_at.as_secs(),
                report.hop_count,
                report.verified,
                report.confirmations,
            ],
        )?;

        Ok(())
    }

    /// Get an intelligence report by ID
    pub fn get(&self, id: Uuid) -> Result<Option<IntelligenceReport>> {
        let conn = self.db.conn();

        let result = conn.query_row(
            "SELECT id, reporter_id, category, danger_level, region,
                    summary, details, urgency, reported_at, expires_at,
                    hop_count, verified, confirmations
             FROM intelligence WHERE id = ?1",
            params![id.as_bytes()],
            |row| self.row_to_intelligence(row),
        );

        match result {
            Ok(report) => Ok(Some(report)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// List all intelligence reports
    pub fn list(&self) -> Result<Vec<IntelligenceReport>> {
        let conn = self.db.conn();

        let mut stmt = conn.prepare(
            "SELECT id, reporter_id, category, danger_level, region,
                    summary, details, urgency, reported_at, expires_at,
                    hop_count, verified, confirmations
             FROM intelligence
             ORDER BY urgency DESC, reported_at DESC"
        )?;

        let reports = stmt.query_map([], |row| self.row_to_intelligence(row))?
            .map(|r| r.map_err(|e| e.into()))
            .collect::<Result<Vec<_>>>()?;

        Ok(reports)
    }

    /// List relevant (non-expired) intelligence reports
    pub fn list_relevant(&self) -> Result<Vec<IntelligenceReport>> {
        let conn = self.db.conn();
        let now = CoarseTimestamp::now().as_secs();

        let mut stmt = conn.prepare(
            "SELECT id, reporter_id, category, danger_level, region,
                    summary, details, urgency, reported_at, expires_at,
                    hop_count, verified, confirmations
             FROM intelligence
             WHERE expires_at > ?1
             ORDER BY urgency DESC, reported_at DESC"
        )?;

        let reports = stmt.query_map([now], |row| self.row_to_intelligence(row))?
            .map(|r| r.map_err(|e| e.into()))
            .collect::<Result<Vec<_>>>()?;

        Ok(reports)
    }

    /// List intelligence reports by category
    pub fn list_by_category(&self, category: IntelligenceCategory) -> Result<Vec<IntelligenceReport>> {
        let conn = self.db.conn();
        let now = CoarseTimestamp::now().as_secs();

        let mut stmt = conn.prepare(
            "SELECT id, reporter_id, category, danger_level, region,
                    summary, details, urgency, reported_at, expires_at,
                    hop_count, verified, confirmations
             FROM intelligence
             WHERE category = ?1 AND expires_at > ?2
             ORDER BY urgency DESC, reported_at DESC"
        )?;

        let reports = stmt.query_map(
            params![category as i32, now],
            |row| self.row_to_intelligence(row)
        )?
        .map(|r| r.map_err(|e| e.into()))
        .collect::<Result<Vec<_>>>()?;

        Ok(reports)
    }

    /// List intelligence reports by region
    pub fn list_by_region(&self, region: &str) -> Result<Vec<IntelligenceReport>> {
        let conn = self.db.conn();
        let now = CoarseTimestamp::now().as_secs();

        let mut stmt = conn.prepare(
            "SELECT id, reporter_id, category, danger_level, region,
                    summary, details, urgency, reported_at, expires_at,
                    hop_count, verified, confirmations
             FROM intelligence
             WHERE region LIKE ?1 AND expires_at > ?2
             ORDER BY urgency DESC, reported_at DESC"
        )?;

        let region_pattern = format!("%{}%", region);
        let reports = stmt.query_map(
            params![region_pattern, now],
            |row| self.row_to_intelligence(row)
        )?
        .map(|r| r.map_err(|e| e.into()))
        .collect::<Result<Vec<_>>>()?;

        Ok(reports)
    }

    /// List high-danger intelligence reports
    pub fn list_high_danger(&self) -> Result<Vec<IntelligenceReport>> {
        let conn = self.db.conn();
        let now = CoarseTimestamp::now().as_secs();

        let mut stmt = conn.prepare(
            "SELECT id, reporter_id, category, danger_level, region,
                    summary, details, urgency, reported_at, expires_at,
                    hop_count, verified, confirmations
             FROM intelligence
             WHERE danger_level >= ?1 AND expires_at > ?2
             ORDER BY danger_level DESC, reported_at DESC"
        )?;

        let reports = stmt.query_map(
            params![DangerLevel::High as i32, now],
            |row| self.row_to_intelligence(row)
        )?
        .map(|r| r.map_err(|e| e.into()))
        .collect::<Result<Vec<_>>>()?;

        Ok(reports)
    }

    /// List verified intelligence reports
    pub fn list_verified(&self) -> Result<Vec<IntelligenceReport>> {
        let conn = self.db.conn();
        let now = CoarseTimestamp::now().as_secs();

        let mut stmt = conn.prepare(
            "SELECT id, reporter_id, category, danger_level, region,
                    summary, details, urgency, reported_at, expires_at,
                    hop_count, verified, confirmations
             FROM intelligence
             WHERE verified = 1 AND expires_at > ?1
             ORDER BY urgency DESC, reported_at DESC"
        )?;

        let reports = stmt.query_map([now], |row| self.row_to_intelligence(row))?
            .map(|r| r.map_err(|e| e.into()))
            .collect::<Result<Vec<_>>>()?;

        Ok(reports)
    }

    /// Add confirmation to a report
    pub fn add_confirmation(&self, id: Uuid) -> Result<()> {
        let conn = self.db.conn();

        conn.execute(
            "UPDATE intelligence
             SET confirmations = confirmations + 1,
                 verified = CASE WHEN confirmations + 1 >= 2 THEN 1 ELSE verified END
             WHERE id = ?1",
            params![id.as_bytes()],
        )?;

        Ok(())
    }

    /// Mark report as verified
    pub fn verify(&self, id: Uuid) -> Result<()> {
        let conn = self.db.conn();

        conn.execute(
            "UPDATE intelligence SET verified = 1 WHERE id = ?1",
            params![id.as_bytes()],
        )?;

        Ok(())
    }

    /// Delete an intelligence report
    pub fn delete(&self, id: Uuid) -> Result<()> {
        let conn = self.db.conn();

        conn.execute(
            "DELETE FROM intelligence WHERE id = ?1",
            params![id.as_bytes()],
        )?;

        Ok(())
    }

    /// Count total intelligence reports
    pub fn count(&self) -> Result<usize> {
        let conn = self.db.conn();

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM intelligence",
            [],
            |row| row.get(0),
        )?;

        Ok(count as usize)
    }

    /// Count relevant (non-expired) reports
    pub fn count_relevant(&self) -> Result<usize> {
        let conn = self.db.conn();
        let now = CoarseTimestamp::now().as_secs();

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM intelligence WHERE expires_at > ?1",
            [now],
            |row| row.get(0),
        )?;

        Ok(count as usize)
    }

    /// Count critical danger reports
    pub fn count_critical_danger(&self) -> Result<usize> {
        let conn = self.db.conn();
        let now = CoarseTimestamp::now().as_secs();

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM intelligence
             WHERE danger_level = ?1 AND expires_at > ?2",
            params![DangerLevel::Critical as i32, now],
            |row| row.get(0),
        )?;

        Ok(count as usize)
    }

    /// Cleanup expired intelligence reports
    pub fn cleanup_expired(&self) -> Result<usize> {
        let conn = self.db.conn();
        let now = CoarseTimestamp::now().as_secs();

        let deleted = conn.execute(
            "DELETE FROM intelligence WHERE expires_at <= ?1",
            [now],
        )?;

        Ok(deleted)
    }

    /// Helper: Convert database row to IntelligenceReport
    fn row_to_intelligence(&self, row: &Row) -> rusqlite::Result<IntelligenceReport> {
        let id_bytes: Vec<u8> = row.get(0)?;
        let id = Uuid::from_slice(&id_bytes).unwrap();

        let reporter_bytes: Vec<u8> = row.get(1)?;
        let reporter = PersonId(Uuid::from_slice(&reporter_bytes).unwrap());

        let category_int: i32 = row.get(2)?;
        let category = match category_int {
            0 => IntelligenceCategory::PoliceActivity,
            1 => IntelligenceCategory::ImmigrationEnforcement,
            2 => IntelligenceCategory::Surveillance,
            3 => IntelligenceCategory::SafeRoute,
            4 => IntelligenceCategory::BorderCrossing,
            5 => IntelligenceCategory::Resource,
            6 => IntelligenceCategory::Compromise,
            7 => IntelligenceCategory::SafetyUpdate,
            8 => IntelligenceCategory::DangerZone,
            9 => IntelligenceCategory::AllClear,
            _ => IntelligenceCategory::SafetyUpdate,
        };

        let danger_level_int: Option<i32> = row.get(3)?;
        let danger_level = danger_level_int.map(|d| match d {
            0 => DangerLevel::None,
            1 => DangerLevel::Low,
            2 => DangerLevel::Moderate,
            3 => DangerLevel::High,
            4 => DangerLevel::Critical,
            _ => DangerLevel::Moderate,
        });

        let region_name: String = row.get(4)?;
        let region = Region::new(region_name);

        let summary: String = row.get(5)?;

        let details_bytes: Option<Vec<u8>> = row.get(6)?;
        let details = details_bytes.map(SecureBytes::new);

        let urgency_int: i32 = row.get(7)?;
        let urgency = match urgency_int {
            1 => Urgency::Low,
            2 => Urgency::Medium,
            3 => Urgency::High,
            4 => Urgency::Critical,
            _ => Urgency::Medium,
        };

        let reported_at_secs: i64 = row.get(8)?;
        let reported_at = CoarseTimestamp::from_datetime(
            chrono::DateTime::from_timestamp(reported_at_secs, 0).unwrap()
        );

        let expires_at_secs: i64 = row.get(9)?;
        let expires_at = CoarseTimestamp::from_datetime(
            chrono::DateTime::from_timestamp(expires_at_secs, 0).unwrap()
        );

        let hop_count: u8 = row.get::<_, i32>(10)? as u8;
        let verified: bool = row.get(11)?;
        let confirmations: u32 = row.get(12)?;

        Ok(IntelligenceReport {
            id,
            reporter,
            category,
            danger_level,
            region,
            summary,
            details,
            urgency,
            reported_at,
            expires_at,
            hop_count,
            verified,
            confirmations,
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

    fn test_intelligence() -> IntelligenceReport {
        IntelligenceReport::new(
            PersonId::new(),
            IntelligenceCategory::PoliceActivity,
            Region::new("Test Region"),
            "Police checkpoint on Main St",
            Urgency::High,
        )
    }

    #[test]
    fn test_save_and_get() {
        let (_tmp, db) = test_db();
        let repo = IntelligenceRepository::new(&db);

        let report = test_intelligence();
        let id = report.id;

        repo.save(&report).unwrap();

        let retrieved = repo.get(id).unwrap();
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, report.id);
        assert_eq!(retrieved.summary, report.summary);
    }

    #[test]
    fn test_list_by_category() {
        let (_tmp, db) = test_db();
        let repo = IntelligenceRepository::new(&db);

        let mut police = test_intelligence();
        police.category = IntelligenceCategory::PoliceActivity;
        repo.save(&police).unwrap();

        let mut resource = test_intelligence();
        resource.category = IntelligenceCategory::Resource;
        repo.save(&resource).unwrap();

        let police_reports = repo.list_by_category(IntelligenceCategory::PoliceActivity).unwrap();
        assert_eq!(police_reports.len(), 1);
        assert_eq!(police_reports[0].id, police.id);
    }

    #[test]
    fn test_list_high_danger() {
        let (_tmp, db) = test_db();
        let repo = IntelligenceRepository::new(&db);

        let critical = test_intelligence().with_danger_level(DangerLevel::Critical);
        repo.save(&critical).unwrap();

        let low = test_intelligence().with_danger_level(DangerLevel::Low);
        repo.save(&low).unwrap();

        let high_danger = repo.list_high_danger().unwrap();
        assert_eq!(high_danger.len(), 1);
        assert_eq!(high_danger[0].id, critical.id);
    }

    #[test]
    fn test_add_confirmation() {
        let (_tmp, db) = test_db();
        let repo = IntelligenceRepository::new(&db);

        let report = test_intelligence();
        let id = report.id;
        assert_eq!(report.confirmations, 0);
        assert!(!report.verified);

        repo.save(&report).unwrap();

        // Add first confirmation
        repo.add_confirmation(id).unwrap();
        let updated = repo.get(id).unwrap().unwrap();
        assert_eq!(updated.confirmations, 1);
        assert!(!updated.verified); // Not yet verified

        // Add second confirmation - should auto-verify
        repo.add_confirmation(id).unwrap();
        let verified = repo.get(id).unwrap().unwrap();
        assert_eq!(verified.confirmations, 2);
        assert!(verified.verified); // Now verified!
    }

    #[test]
    fn test_verify() {
        let (_tmp, db) = test_db();
        let repo = IntelligenceRepository::new(&db);

        let report = test_intelligence();
        let id = report.id;
        repo.save(&report).unwrap();

        repo.verify(id).unwrap();

        let verified = repo.get(id).unwrap().unwrap();
        assert!(verified.verified);
    }

    #[test]
    fn test_list_verified() {
        let (_tmp, db) = test_db();
        let repo = IntelligenceRepository::new(&db);

        let mut verified = test_intelligence();
        verified.verified = true;
        repo.save(&verified).unwrap();

        let unverified = test_intelligence();
        repo.save(&unverified).unwrap();

        let verified_list = repo.list_verified().unwrap();
        assert_eq!(verified_list.len(), 1);
        assert_eq!(verified_list[0].id, verified.id);
    }

    #[test]
    fn test_count() {
        let (_tmp, db) = test_db();
        let repo = IntelligenceRepository::new(&db);

        assert_eq!(repo.count().unwrap(), 0);

        repo.save(&test_intelligence()).unwrap();
        assert_eq!(repo.count().unwrap(), 1);

        repo.save(&test_intelligence()).unwrap();
        assert_eq!(repo.count().unwrap(), 2);
    }

    #[test]
    fn test_delete() {
        let (_tmp, db) = test_db();
        let repo = IntelligenceRepository::new(&db);

        let report = test_intelligence();
        let id = report.id;
        repo.save(&report).unwrap();

        repo.delete(id).unwrap();

        assert!(repo.get(id).unwrap().is_none());
    }
}
