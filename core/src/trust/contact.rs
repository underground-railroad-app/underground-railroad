//! Contact information and management

use crate::{CoarseTimestamp, Fingerprint, PersonId, Region, SecureBytes, TrustLevel};
use serde::{Deserialize, Serialize};

/// A contact in the trust network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    /// Unique ID for this person
    pub id: PersonId,

    /// Contact information
    pub info: ContactInfo,

    /// Trust level
    pub trust_level: TrustLevel,

    /// Public key fingerprint (for verification)
    pub fingerprint: Fingerprint,

    /// Veilid route information (encrypted)
    pub veilid_route: SecureBytes,

    /// When was this contact added?
    pub added_at: CoarseTimestamp,

    /// Last time we communicated
    pub last_contact: Option<CoarseTimestamp>,

    /// Who introduced us to this contact?
    pub introduced_by: Option<PersonId>,

    /// Private notes about this contact (encrypted)
    pub notes: Option<SecureBytes>,

    /// Is this contact currently available?
    pub available: bool,

    /// Tags for categorizing contacts
    pub tags: Vec<String>,
}

/// Contact information (what we know about them)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactInfo {
    /// Display name (user-chosen, can be pseudonym)
    pub name: String,

    /// Approximate region (if known)
    pub region: Option<Region>,

    /// Avatar/photo hash (optional, for recognition)
    pub avatar_hash: Option<[u8; 32]>,

    /// Languages they speak
    pub languages: Vec<String>,

    /// What capabilities do they offer?
    pub capabilities: Vec<Capability>,
}

/// Capabilities a contact might have
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Capability {
    /// Can offer safe house
    SafeHouse,

    /// Can provide transportation
    Transportation,

    /// Can provide medical help
    Medical,

    /// Can provide legal help
    Legal,

    /// Can provide financial help
    Financial,

    /// Can provide supplies (food, clothing, etc.)
    Supplies,

    /// Can help with documents
    Documents,

    /// Can translate/interpret
    Translation,

    /// Can relay messages
    Relay,

    /// Has intelligence/local knowledge
    Intelligence,
}

impl Contact {
    /// Create a new contact
    pub fn new(
        id: PersonId,
        name: impl Into<String>,
        fingerprint: Fingerprint,
        veilid_route: SecureBytes,
        trust_level: TrustLevel,
    ) -> Self {
        let now = CoarseTimestamp::now();

        Self {
            id,
            info: ContactInfo {
                name: name.into(),
                region: None,
                avatar_hash: None,
                languages: Vec::new(),
                capabilities: Vec::new(),
            },
            trust_level,
            fingerprint,
            veilid_route,
            added_at: now,
            last_contact: None,
            introduced_by: None,
            notes: None,
            available: true,
            tags: Vec::new(),
        }
    }

    /// Update last contact time
    pub fn update_last_contact(&mut self) {
        self.last_contact = Some(CoarseTimestamp::now());
    }

    /// Add a capability
    pub fn add_capability(&mut self, capability: Capability) {
        if !self.info.capabilities.contains(&capability) {
            self.info.capabilities.push(capability);
        }
    }

    /// Check if contact has a specific capability
    pub fn has_capability(&self, capability: Capability) -> bool {
        self.info.capabilities.contains(&capability)
    }

    /// Add a tag
    pub fn add_tag(&mut self, tag: impl Into<String>) {
        let tag = tag.into();
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    /// Check if contact has a tag
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }

    /// Update trust level
    pub fn update_trust(&mut self, new_level: TrustLevel) {
        self.trust_level = new_level;
    }

    /// Check if we can see this contact's activity
    pub fn can_see_activity(&self) -> bool {
        self.trust_level.can_see_activity()
    }

    /// Check if this contact can relay messages
    pub fn can_relay_messages(&self) -> bool {
        self.trust_level.can_relay()
    }

    /// Get verification words for this contact
    pub fn verification_words(&self) -> [&'static str; 3] {
        self.fingerprint.to_words()
    }
}

impl ContactInfo {
    /// Create new contact info
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            region: None,
            avatar_hash: None,
            languages: Vec::new(),
            capabilities: Vec::new(),
        }
    }

    /// Add a language
    pub fn add_language(&mut self, lang: impl Into<String>) {
        let lang = lang.into();
        if !self.languages.contains(&lang) {
            self.languages.push(lang);
        }
    }

    /// Check if contact speaks a language
    pub fn speaks(&self, lang: &str) -> bool {
        self.languages.iter().any(|l| l == lang)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contact_creation() {
        let id = PersonId::new();
        let fingerprint = Fingerprint::new([0u8; 32]);
        let route = SecureBytes::new(vec![1, 2, 3, 4]);

        let contact = Contact::new(
            id,
            "Alice",
            fingerprint,
            route,
            TrustLevel::VerifiedInPerson,
        );

        assert_eq!(contact.info.name, "Alice");
        assert_eq!(contact.trust_level, TrustLevel::VerifiedInPerson);
        assert!(contact.can_see_activity());
    }

    #[test]
    fn test_contact_capabilities() {
        let mut contact = Contact::new(
            PersonId::new(),
            "Bob",
            Fingerprint::new([0u8; 32]),
            SecureBytes::new(vec![]),
            TrustLevel::VerifiedRemote,
        );

        assert!(!contact.has_capability(Capability::SafeHouse));

        contact.add_capability(Capability::SafeHouse);
        contact.add_capability(Capability::Transportation);

        assert!(contact.has_capability(Capability::SafeHouse));
        assert!(contact.has_capability(Capability::Transportation));
        assert!(!contact.has_capability(Capability::Medical));
    }

    #[test]
    fn test_contact_tags() {
        let mut contact = Contact::new(
            PersonId::new(),
            "Charlie",
            Fingerprint::new([0u8; 32]),
            SecureBytes::new(vec![]),
            TrustLevel::Introduced,
        );

        contact.add_tag("trusted");
        contact.add_tag("driver");

        assert!(contact.has_tag("trusted"));
        assert!(contact.has_tag("driver"));
        assert!(!contact.has_tag("unknown"));
    }

    #[test]
    fn test_contact_languages() {
        let mut info = ContactInfo::new("David");

        info.add_language("en");
        info.add_language("es");
        info.add_language("zh");

        assert!(info.speaks("en"));
        assert!(info.speaks("es"));
        assert!(info.speaks("zh"));
        assert!(!info.speaks("fr"));
    }
}
