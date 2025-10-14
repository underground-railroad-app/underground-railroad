//! Trust and web-of-trust implementation
//!
//! The web of trust is critical for the Underground Railroad:
//! - Verify people before trusting them
//! - Vouch for others you trust
//! - Build a decentralized trust network
//! - Prevent infiltration by adversaries

pub mod contact;
pub mod graph;
pub mod verification;

pub use contact::{Contact, ContactInfo, Capability};
pub use graph::{TrustGraph, TrustPath};
pub use verification::{VerificationMethod, VerificationProof};

use crate::{Fingerprint, PersonId, TrustLevel};
use serde::{Deserialize, Serialize};

/// A trust relationship between two people
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustRelationship {
    /// Who is trusting (local user)
    pub truster: PersonId,

    /// Who is being trusted
    pub trustee: PersonId,

    /// Level of trust
    pub level: TrustLevel,

    /// How was this relationship established?
    pub verification: VerificationMethod,

    /// When was trust established?
    pub established_at: crate::CoarseTimestamp,

    /// When was trust last updated?
    pub updated_at: crate::CoarseTimestamp,

    /// Who introduced this person? (if applicable)
    pub introduced_by: Option<PersonId>,

    /// Notes about this relationship (encrypted)
    pub notes: Option<crate::SecureBytes>,
}

impl TrustRelationship {
    /// Create a new trust relationship
    pub fn new(
        truster: PersonId,
        trustee: PersonId,
        level: TrustLevel,
        verification: VerificationMethod,
    ) -> Self {
        let now = crate::CoarseTimestamp::now();

        Self {
            truster,
            trustee,
            level,
            verification,
            established_at: now,
            updated_at: now,
            introduced_by: None,
            notes: None,
        }
    }

    /// Update trust level
    pub fn update_level(&mut self, new_level: TrustLevel) {
        self.level = new_level;
        self.updated_at = crate::CoarseTimestamp::now();
    }

    /// Set who introduced this person
    pub fn set_introducer(&mut self, introducer: PersonId) {
        self.introduced_by = Some(introducer);
        self.updated_at = crate::CoarseTimestamp::now();
    }

    /// Check if this relationship allows seeing activity
    pub fn allows_activity(&self) -> bool {
        self.level.can_see_activity()
    }

    /// Check if this relationship allows message relaying
    pub fn allows_relay(&self) -> bool {
        self.level.can_relay()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trust_relationship() {
        let alice = PersonId::new();
        let bob = PersonId::new();

        let mut trust = TrustRelationship::new(
            alice,
            bob,
            TrustLevel::VerifiedRemote,
            VerificationMethod::VideoCall,
        );

        assert_eq!(trust.level, TrustLevel::VerifiedRemote);
        assert!(trust.allows_activity());
        assert!(trust.allows_relay());

        trust.update_level(TrustLevel::VerifiedInPerson);
        assert_eq!(trust.level, TrustLevel::VerifiedInPerson);
    }
}
