//! Database repository layer
//!
//! Provides high-level API for database operations.
//! Abstracts away SQL queries and provides type-safe operations.

pub mod contacts;
pub mod safe_houses;
pub mod emergencies;
pub mod intelligence;
pub mod identity;
pub mod messages;

pub use contacts::ContactRepository;
pub use safe_houses::SafeHouseRepository;
pub use emergencies::EmergencyRepository;
pub use intelligence::IntelligenceRepository;
pub use identity::IdentityRepository;
pub use messages::{MessageRepository, MessageDirection};

use crate::{Result, storage::Database};

/// Repository manager - provides access to all repositories
pub struct RepositoryManager<'db> {
    contacts: ContactRepository<'db>,
    safe_houses: SafeHouseRepository<'db>,
    emergencies: EmergencyRepository<'db>,
    intelligence: IntelligenceRepository<'db>,
    identity: IdentityRepository<'db>,
    messages: MessageRepository<'db>,
}

impl<'db> RepositoryManager<'db> {
    /// Create a new repository manager
    pub fn new(db: &'db Database) -> Self {
        Self {
            contacts: ContactRepository::new(db),
            safe_houses: SafeHouseRepository::new(db),
            emergencies: EmergencyRepository::new(db),
            intelligence: IntelligenceRepository::new(db),
            identity: IdentityRepository::new(db),
            messages: MessageRepository::new(db),
        }
    }

    /// Get contacts repository
    pub fn contacts(&self) -> &ContactRepository<'db> {
        &self.contacts
    }

    /// Get safe houses repository
    pub fn safe_houses(&self) -> &SafeHouseRepository<'db> {
        &self.safe_houses
    }

    /// Get emergencies repository
    pub fn emergencies(&self) -> &EmergencyRepository<'db> {
        &self.emergencies
    }

    /// Get intelligence repository
    pub fn intelligence(&self) -> &IntelligenceRepository<'db> {
        &self.intelligence
    }

    /// Get identity repository
    pub fn identity(&self) -> &IdentityRepository<'db> {
        &self.identity
    }

    /// Get messages repository
    pub fn messages(&self) -> &MessageRepository<'db> {
        &self.messages
    }
}
