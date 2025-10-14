//! Persona management - multiple isolated identities

use super::Identity;
use crate::{Fingerprint, PersonId, Result};
use std::collections::HashMap;

/// Manager for multiple personas
#[derive(Debug)]
pub struct PersonaManager {
    /// All personas (ID -> Identity)
    personas: HashMap<PersonId, Identity>,

    /// Currently active persona
    active: Option<PersonId>,

    /// Primary persona ID
    primary: Option<PersonId>,
}

impl PersonaManager {
    /// Create a new empty persona manager
    pub fn new() -> Self {
        Self {
            personas: HashMap::new(),
            active: None,
            primary: None,
        }
    }

    /// Create a new persona
    pub fn create_persona(&mut self, name: impl Into<String>, is_primary: bool) -> Result<PersonId> {
        let identity = Identity::generate(name, is_primary)?;
        let id = identity.id;

        if is_primary {
            self.primary = Some(id);

            // If this is the first persona, make it active
            if self.active.is_none() {
                self.active = Some(id);
            }
        }

        self.personas.insert(id, identity);

        Ok(id)
    }

    /// Add an existing identity as a persona
    pub fn add_persona(&mut self, identity: Identity) -> Result<PersonId> {
        let id = identity.id;

        if identity.is_primary {
            self.primary = Some(id);

            if self.active.is_none() {
                self.active = Some(id);
            }
        }

        self.personas.insert(id, identity);

        Ok(id)
    }

    /// Get a persona by ID
    pub fn get_persona(&self, id: PersonId) -> Option<&Identity> {
        self.personas.get(&id)
    }

    /// Get a mutable reference to a persona
    pub fn get_persona_mut(&mut self, id: PersonId) -> Option<&mut Identity> {
        self.personas.get_mut(&id)
    }

    /// Get the active persona
    pub fn active_persona(&self) -> Option<&Identity> {
        self.active.and_then(|id| self.personas.get(&id))
    }

    /// Get the primary persona
    pub fn primary_persona(&self) -> Option<&Identity> {
        self.primary.and_then(|id| self.personas.get(&id))
    }

    /// Switch to a different persona
    pub fn switch_persona(&mut self, id: PersonId) -> Result<()> {
        if !self.personas.contains_key(&id) {
            return Err(crate::Error::NotFound("Persona not found".to_string()));
        }

        self.active = Some(id);
        Ok(())
    }

    /// List all personas
    pub fn list_personas(&self) -> Vec<PersonaSummary> {
        self.personas
            .values()
            .map(|identity| PersonaSummary {
                id: identity.id,
                name: identity.name.clone(),
                fingerprint: identity.fingerprint.clone(),
                is_primary: identity.is_primary,
                is_active: Some(identity.id) == self.active,
            })
            .collect()
    }

    /// Delete a persona
    pub fn delete_persona(&mut self, id: PersonId) -> Result<()> {
        // Can't delete primary persona
        if Some(id) == self.primary {
            return Err(crate::Error::PermissionDenied(
                "Cannot delete primary persona".to_string(),
            ));
        }

        // Can't delete active persona
        if Some(id) == self.active {
            return Err(crate::Error::PermissionDenied(
                "Cannot delete active persona (switch first)".to_string(),
            ));
        }

        self.personas
            .remove(&id)
            .ok_or_else(|| crate::Error::NotFound("Persona not found".to_string()))?;

        Ok(())
    }

    /// Get total number of personas
    pub fn count(&self) -> usize {
        self.personas.len()
    }

    /// Check if a persona exists
    pub fn has_persona(&self, id: PersonId) -> bool {
        self.personas.contains_key(&id)
    }
}

impl Default for PersonaManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary information about a persona
#[derive(Debug, Clone)]
pub struct PersonaSummary {
    pub id: PersonId,
    pub name: String,
    pub fingerprint: Fingerprint,
    pub is_primary: bool,
    pub is_active: bool,
}

/// Builder for creating a new persona
pub struct PersonaBuilder {
    name: String,
    is_primary: bool,
    from_seed: Option<[u8; 32]>,
}

impl PersonaBuilder {
    /// Create a new persona builder
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            is_primary: false,
            from_seed: None,
        }
    }

    /// Mark this as the primary persona
    pub fn primary(mut self) -> Self {
        self.is_primary = true;
        self
    }

    /// Create from a specific seed (for recovery)
    pub fn from_seed(mut self, seed: [u8; 32]) -> Self {
        self.from_seed = Some(seed);
        self
    }

    /// Build the identity
    pub fn build(self) -> Result<Identity> {
        if let Some(seed) = self.from_seed {
            Identity::from_seed(self.name, &seed, self.is_primary)
        } else {
            Identity::generate(self.name, self.is_primary)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_persona_manager_creation() {
        let manager = PersonaManager::new();
        assert_eq!(manager.count(), 0);
        assert!(manager.active_persona().is_none());
        assert!(manager.primary_persona().is_none());
    }

    #[test]
    fn test_create_persona() {
        let mut manager = PersonaManager::new();

        let id = manager.create_persona("Alice", true).unwrap();

        assert_eq!(manager.count(), 1);
        assert!(manager.has_persona(id));
        assert_eq!(manager.active_persona().unwrap().name, "Alice");
        assert_eq!(manager.primary_persona().unwrap().name, "Alice");
    }

    #[test]
    fn test_multiple_personas() {
        let mut manager = PersonaManager::new();

        let alice_id = manager.create_persona("Alice", true).unwrap();
        let bob_id = manager.create_persona("Bob", false).unwrap();
        let charlie_id = manager.create_persona("Charlie", false).unwrap();

        assert_eq!(manager.count(), 3);

        // Alice should be active (first one)
        assert_eq!(manager.active_persona().unwrap().name, "Alice");

        // Switch to Bob
        manager.switch_persona(bob_id).unwrap();
        assert_eq!(manager.active_persona().unwrap().name, "Bob");

        // Alice should still be primary
        assert_eq!(manager.primary_persona().unwrap().name, "Alice");
    }

    #[test]
    fn test_delete_persona() {
        let mut manager = PersonaManager::new();

        let alice_id = manager.create_persona("Alice", true).unwrap();
        let bob_id = manager.create_persona("Bob", false).unwrap();

        // Can't delete primary
        assert!(manager.delete_persona(alice_id).is_err());

        // Can't delete active (Alice is active)
        assert!(manager.delete_persona(alice_id).is_err());

        // Switch to Bob
        manager.switch_persona(bob_id).unwrap();

        // Still can't delete primary
        assert!(manager.delete_persona(alice_id).is_err());

        // Can delete Bob now (not primary, not active)
        // Wait, Bob IS active now, so can't delete
        assert!(manager.delete_persona(bob_id).is_err());
    }

    #[test]
    fn test_list_personas() {
        let mut manager = PersonaManager::new();

        manager.create_persona("Alice", true).unwrap();
        manager.create_persona("Bob", false).unwrap();
        manager.create_persona("Charlie", false).unwrap();

        let list = manager.list_personas();
        assert_eq!(list.len(), 3);

        // One should be primary
        assert_eq!(list.iter().filter(|p| p.is_primary).count(), 1);

        // One should be active
        assert_eq!(list.iter().filter(|p| p.is_active).count(), 1);
    }

    #[test]
    fn test_persona_builder() {
        let persona = PersonaBuilder::new("Test User")
            .primary()
            .build()
            .unwrap();

        assert_eq!(persona.name, "Test User");
        assert!(persona.is_primary);
    }

    #[test]
    fn test_persona_builder_from_seed() {
        let seed = [42u8; 32];

        let persona1 = PersonaBuilder::new("Test")
            .from_seed(seed)
            .build()
            .unwrap();

        let persona2 = PersonaBuilder::new("Test")
            .from_seed(seed)
            .build()
            .unwrap();

        // Same seed = same keys (but different IDs)
        assert_eq!(persona1.public_key_bytes(), persona2.public_key_bytes());
        assert_ne!(persona1.id, persona2.id);
    }

    #[test]
    fn test_switch_invalid_persona() {
        let mut manager = PersonaManager::new();

        let fake_id = PersonId::new();
        assert!(manager.switch_persona(fake_id).is_err());
    }
}
