//! Intelligence reports - real-time information about safe/dangerous areas

use crate::{CoarseTimestamp, PersonId, Region, SecureBytes, Urgency};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Intelligence report - information about safety/danger in an area
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceReport {
    /// Unique ID for this report
    pub id: Uuid,

    /// Who submitted this report
    pub reporter: PersonId,

    /// What category is this?
    pub category: IntelligenceCategory,

    /// Danger level (if applicable)
    pub danger_level: Option<DangerLevel>,

    /// Which region does this affect?
    pub region: Region,

    /// Short summary
    pub summary: String,

    /// Detailed information (encrypted)
    pub details: Option<SecureBytes>,

    /// How urgent is this information?
    pub urgency: Urgency,

    /// When was this reported?
    pub reported_at: CoarseTimestamp,

    /// When does this expire/become outdated?
    pub expires_at: CoarseTimestamp,

    /// How many hops has this propagated?
    pub hop_count: u8,

    /// Has this been verified by multiple sources?
    pub verified: bool,

    /// Number of confirmations from other sources
    pub confirmations: u32,
}

/// Category of intelligence report
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntelligenceCategory {
    /// Police checkpoint or raid
    PoliceActivity,

    /// ICE/immigration enforcement
    ImmigrationEnforcement,

    /// Surveillance detected
    Surveillance,

    /// Safe route confirmed
    SafeRoute,

    /// Border crossing status
    BorderCrossing,

    /// Resource availability (legal aid, medical, etc.)
    Resource,

    /// Safe house compromised
    Compromise,

    /// General safety update
    SafetyUpdate,

    /// Dangerous area warning
    DangerZone,

    /// All-clear / area is safe
    AllClear,
}

/// Level of danger
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DangerLevel {
    /// No danger
    None = 0,

    /// Low risk
    Low = 1,

    /// Moderate risk - be cautious
    Moderate = 2,

    /// High risk - avoid if possible
    High = 3,

    /// Critical - do not enter, immediate danger
    Critical = 4,
}

impl IntelligenceReport {
    /// Create a new intelligence report
    pub fn new(
        reporter: PersonId,
        category: IntelligenceCategory,
        region: Region,
        summary: impl Into<String>,
        urgency: Urgency,
    ) -> Self {
        let now = CoarseTimestamp::now();
        let expires_at = CoarseTimestamp::from_datetime(
            chrono::Utc::now() + Self::default_expiry(category, urgency)
        );

        Self {
            id: Uuid::new_v4(),
            reporter,
            category,
            danger_level: None,
            region,
            summary: summary.into(),
            details: None,
            urgency,
            reported_at: now,
            expires_at,
            hop_count: 0,
            verified: false,
            confirmations: 0,
        }
    }

    /// Default expiry time based on category and urgency
    fn default_expiry(category: IntelligenceCategory, urgency: Urgency) -> chrono::Duration {
        match category {
            // Police/enforcement activity is time-sensitive
            IntelligenceCategory::PoliceActivity |
            IntelligenceCategory::ImmigrationEnforcement => {
                match urgency {
                    Urgency::Critical => chrono::Duration::hours(2),
                    Urgency::High => chrono::Duration::hours(6),
                    _ => chrono::Duration::hours(12),
                }
            }

            // Surveillance reports stay relevant longer
            IntelligenceCategory::Surveillance |
            IntelligenceCategory::DangerZone => chrono::Duration::days(3),

            // Resources and safe routes stay relevant longest
            IntelligenceCategory::Resource |
            IntelligenceCategory::SafeRoute => chrono::Duration::days(30),

            // Compromises never expire (permanent warning)
            IntelligenceCategory::Compromise => chrono::Duration::days(365),

            // All-clear expires quickly
            IntelligenceCategory::AllClear => chrono::Duration::hours(24),

            // Default
            _ => chrono::Duration::days(7),
        }
    }

    /// Check if this report has expired
    pub fn is_expired(&self) -> bool {
        self.expires_at.is_expired(chrono::Duration::zero())
    }

    /// Check if this report is still relevant
    pub fn is_relevant(&self) -> bool {
        !self.is_expired()
    }

    /// Add a confirmation from another source
    pub fn add_confirmation(&mut self) {
        self.confirmations += 1;

        // Auto-verify after multiple confirmations
        if self.confirmations >= 2 {
            self.verified = true;
        }
    }

    /// Increment hop count (for propagation tracking)
    pub fn increment_hops(&mut self) {
        self.hop_count += 1;
    }

    /// Should this report propagate further?
    pub fn should_propagate(&self) -> bool {
        // Don't propagate if expired
        if self.is_expired() {
            return false;
        }

        // Check against max hops for this urgency
        self.hop_count < self.urgency.propagation_hops()
    }

    /// Calculate priority score for display (higher = show first)
    pub fn priority_score(&self) -> u32 {
        let urgency_score = self.urgency as u32 * 1000;

        let danger_score = self.danger_level
            .map(|d| d as u32 * 500)
            .unwrap_or(0);

        let verified_bonus = if self.verified { 200 } else { 0 };

        let recency_score = {
            let age_hours = (CoarseTimestamp::now().as_secs() - self.reported_at.as_secs()) / 3600;
            // Newer reports score higher
            100u32.saturating_sub(age_hours.min(100) as u32)
        };

        urgency_score + danger_score + verified_bonus + recency_score
    }

    /// Set danger level
    pub fn with_danger_level(mut self, level: DangerLevel) -> Self {
        self.danger_level = Some(level);
        self
    }

    /// Add detailed information
    pub fn with_details(mut self, details: SecureBytes) -> Self {
        self.details = Some(details);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intelligence_report() {
        let reporter = PersonId::new();
        let region = Region::new("Downtown");

        let report = IntelligenceReport::new(
            reporter,
            IntelligenceCategory::PoliceActivity,
            region,
            "Checkpoint on Main St",
            Urgency::High,
        );

        assert!(report.is_relevant());
        assert!(!report.is_expired());
        assert_eq!(report.hop_count, 0);
    }

    #[test]
    fn test_intelligence_propagation() {
        let reporter = PersonId::new();
        let region = Region::new("Test");

        let mut critical = IntelligenceReport::new(
            reporter,
            IntelligenceCategory::PoliceActivity,
            region.clone(),
            "Raid in progress",
            Urgency::Critical,
        );

        assert!(critical.should_propagate());

        // Simulate propagation
        for _ in 0..5 {
            critical.increment_hops();
        }

        // Critical has 5 hops max
        assert!(!critical.should_propagate());

        let mut low = IntelligenceReport::new(
            reporter,
            IntelligenceCategory::Resource,
            region,
            "Legal aid available",
            Urgency::Low,
        );

        // Low only propagates 1 hop
        low.increment_hops();
        assert!(!low.should_propagate());
    }

    #[test]
    fn test_intelligence_verification() {
        let reporter = PersonId::new();
        let region = Region::new("Test");

        let mut report = IntelligenceReport::new(
            reporter,
            IntelligenceCategory::PoliceActivity,
            region,
            "Checkpoint reported",
            Urgency::High,
        );

        assert!(!report.verified);

        report.add_confirmation();
        assert!(!report.verified);  // Need 2 confirmations

        report.add_confirmation();
        assert!(report.verified);  // Now verified
    }

    #[test]
    fn test_priority_scoring() {
        let reporter = PersonId::new();
        let region = Region::new("Test");

        let critical = IntelligenceReport::new(
            reporter,
            IntelligenceCategory::DangerZone,
            region.clone(),
            "Critical danger",
            Urgency::Critical,
        ).with_danger_level(DangerLevel::Critical);

        let low = IntelligenceReport::new(
            reporter,
            IntelligenceCategory::Resource,
            region,
            "Info available",
            Urgency::Low,
        );

        assert!(critical.priority_score() > low.priority_score());
    }
}
