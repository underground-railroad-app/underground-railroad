//! Emergency request handling - for people in immediate danger

use crate::{CoarseTimestamp, EmergencyId, PersonId, Region, SecureBytes, Urgency};
use serde::{Deserialize, Serialize};

/// Emergency request - someone needs immediate help
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyRequest {
    /// Unique ID for this emergency
    pub id: EmergencyId,

    /// Who is requesting help (may be anonymous)
    pub requester: Option<PersonId>,

    /// What kind of help is needed
    pub needs: Vec<EmergencyNeed>,

    /// Approximate location (coarse, not exact)
    pub region: Region,

    /// How urgent is this?
    pub urgency: Urgency,

    /// How many people need help?
    pub num_people: u32,

    /// Number of children
    pub num_children: u32,

    /// Special requirements (encrypted)
    pub special_needs: Option<SecureBytes>,

    /// Additional context (encrypted)
    pub notes: Option<SecureBytes>,

    /// When was this request created?
    pub created_at: CoarseTimestamp,

    /// When does this request expire?
    pub expires_at: CoarseTimestamp,

    /// Current status
    pub status: EmergencyStatus,
}

/// Types of emergency assistance needed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmergencyNeed {
    /// Need place to hide/stay immediately
    SafeShelter,

    /// Need transportation out of danger area
    Transportation,

    /// Being followed/watched
    Surveillance,

    /// Medical emergency
    Medical,

    /// Need food/water
    Supplies,

    /// Need cash/financial help
    Financial,

    /// Need legal assistance
    Legal,

    /// Police/authorities present
    ImmediateDanger,

    /// Other (details in notes)
    Other,
}

/// Status of an emergency request
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmergencyStatus {
    /// Active - still need help
    Active,

    /// Someone responded and is helping
    InProgress,

    /// Resolved - safe now
    Resolved,

    /// Expired - no longer relevant
    Expired,

    /// Cancelled by requester
    Cancelled,
}

/// Response to an emergency request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyResponse {
    /// Which emergency is this responding to?
    pub emergency_id: EmergencyId,

    /// Who is responding?
    pub responder: PersonId,

    /// What can they provide?
    pub can_provide: Vec<EmergencyNeed>,

    /// How soon can they help?
    pub estimated_time_mins: Option<u32>,

    /// Distance from requester (approximate)
    pub distance_km: Option<f64>,

    /// Additional information (encrypted)
    pub details: Option<SecureBytes>,

    /// When was this response sent?
    pub responded_at: CoarseTimestamp,
}

impl EmergencyRequest {
    /// Create a new emergency request
    pub fn new(
        requester: Option<PersonId>,
        needs: Vec<EmergencyNeed>,
        region: Region,
        urgency: Urgency,
        num_people: u32,
    ) -> Self {
        let now = CoarseTimestamp::now();
        let expires_at = CoarseTimestamp::from_datetime(
            chrono::Utc::now() + urgency.default_expiry()
        );

        Self {
            id: EmergencyId::new(),
            requester,
            needs,
            region,
            urgency,
            num_people,
            num_children: 0,
            special_needs: None,
            notes: None,
            created_at: now,
            expires_at,
            status: EmergencyStatus::Active,
        }
    }

    /// Check if this emergency has expired
    pub fn is_expired(&self) -> bool {
        self.expires_at.is_expired(chrono::Duration::zero())
    }

    /// Check if this emergency is still active
    pub fn is_active(&self) -> bool {
        matches!(self.status, EmergencyStatus::Active | EmergencyStatus::InProgress)
            && !self.is_expired()
    }

    /// Mark this emergency as resolved
    pub fn resolve(&mut self) {
        self.status = EmergencyStatus::Resolved;
    }

    /// Mark this emergency as in progress
    pub fn start_helping(&mut self) {
        if self.status == EmergencyStatus::Active {
            self.status = EmergencyStatus::InProgress;
        }
    }

    /// Calculate priority score for sorting (higher = more urgent)
    pub fn priority_score(&self) -> u32 {
        let urgency_score = self.urgency as u32 * 1000;
        let age_mins = (CoarseTimestamp::now().as_secs() - self.created_at.as_secs()) / 60;
        let age_score = (age_mins as u32).min(100);  // Cap at 100

        // Immediate danger gets highest priority
        let danger_bonus = if self.needs.contains(&EmergencyNeed::ImmediateDanger) {
            5000
        } else {
            0
        };

        urgency_score + age_score + danger_bonus
    }
}

impl EmergencyResponse {
    /// Create a new emergency response
    pub fn new(
        emergency_id: EmergencyId,
        responder: PersonId,
        can_provide: Vec<EmergencyNeed>,
    ) -> Self {
        Self {
            emergency_id,
            responder,
            can_provide,
            estimated_time_mins: None,
            distance_km: None,
            details: None,
            responded_at: CoarseTimestamp::now(),
        }
    }

    /// Add estimated time to arrive
    pub fn with_eta(mut self, mins: u32) -> Self {
        self.estimated_time_mins = Some(mins);
        self
    }

    /// Add distance information
    pub fn with_distance(mut self, km: f64) -> Self {
        self.distance_km = Some(km);
        self
    }

    /// Add encrypted details
    pub fn with_details(mut self, details: SecureBytes) -> Self {
        self.details = Some(details);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emergency_request_creation() {
        let region = Region::new("Test Area");
        let request = EmergencyRequest::new(
            None,
            vec![EmergencyNeed::SafeShelter],
            region,
            Urgency::Critical,
            2,
        );

        assert!(request.is_active());
        assert!(!request.is_expired());
        assert_eq!(request.needs.len(), 1);
        assert_eq!(request.status, EmergencyStatus::Active);
    }

    #[test]
    fn test_emergency_priority() {
        let region = Region::new("Test");

        let critical = EmergencyRequest::new(
            None,
            vec![EmergencyNeed::ImmediateDanger],
            region.clone(),
            Urgency::Critical,
            1,
        );

        let low = EmergencyRequest::new(
            None,
            vec![EmergencyNeed::Supplies],
            region,
            Urgency::Low,
            1,
        );

        assert!(critical.priority_score() > low.priority_score());
    }

    #[test]
    fn test_emergency_response() {
        let response = EmergencyResponse::new(
            EmergencyId::new(),
            PersonId::new(),
            vec![EmergencyNeed::SafeShelter, EmergencyNeed::Supplies],
        )
        .with_eta(15)
        .with_distance(5.0);

        assert_eq!(response.estimated_time_mins, Some(15));
        assert_eq!(response.distance_km, Some(5.0));
    }
}
