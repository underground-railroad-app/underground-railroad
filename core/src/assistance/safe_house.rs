//! Safe house registry and management

use crate::{CoarseTimestamp, PersonId, Region, SafeHouseId, SecureBytes};
use serde::{Deserialize, Serialize};

/// A safe house - a place where people can hide and be protected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafeHouse {
    /// Unique ID for this safe house
    pub id: SafeHouseId,

    /// Who operates this safe house
    pub operator: PersonId,

    /// Name/code name for this location (e.g., "Green House")
    pub name: String,

    /// Approximate region (NOT exact address)
    pub region: Region,

    /// What capabilities does this safe house have?
    pub capabilities: Vec<SafeHouseCapability>,

    /// How many people can stay?
    pub capacity: u32,

    /// How many people are currently here?
    pub current_occupancy: u32,

    /// Availability status
    pub status: SafeHouseStatus,

    /// Special accommodations available
    pub accommodations: Vec<Accommodation>,

    /// How long can people stay? (days)
    pub max_stay_days: Option<u32>,

    /// Additional notes (encrypted)
    pub notes: Option<SecureBytes>,

    /// Trust/verification level
    pub verified: bool,

    /// When was this safe house registered?
    pub registered_at: CoarseTimestamp,

    /// When was this information last updated?
    pub updated_at: CoarseTimestamp,
}

/// What a safe house can provide
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SafeHouseCapability {
    /// Place to sleep
    Shelter,

    /// Food and water
    Food,

    /// Medical supplies/first aid
    Medical,

    /// Internet/phone access
    Communication,

    /// Cash assistance
    Financial,

    /// Can provide transportation
    Transportation,

    /// Legal advice/assistance
    Legal,

    /// Document assistance (photos, copies, etc.)
    Documents,

    /// Child care
    ChildCare,

    /// Translation services
    Translation,

    /// Long-term housing (weeks/months)
    LongTerm,
}

/// Availability status of a safe house
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SafeHouseStatus {
    /// Available right now
    Available,

    /// Available with advance notice (hours)
    AvailableWithNotice,

    /// Currently occupied, not available
    Occupied,

    /// Temporarily unavailable (compromised, under surveillance, etc.)
    TemporarilyUnavailable,

    /// Permanently closed
    Closed,
}

/// Special accommodations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Accommodation {
    /// Wheelchair accessible
    WheelchairAccessible,

    /// Suitable for families with children
    FamilyFriendly,

    /// Pet-friendly
    PetFriendly,

    /// Separate rooms for privacy
    PrivateRooms,

    /// Ground floor (no stairs)
    GroundFloor,

    /// Has washing machine/shower
    Facilities,

    /// Quiet/secluded location
    Secluded,

    /// Near public transportation
    TransitAccessible,
}

impl SafeHouse {
    /// Create a new safe house registry entry
    pub fn new(
        operator: PersonId,
        name: impl Into<String>,
        region: Region,
        capacity: u32,
    ) -> Self {
        let now = CoarseTimestamp::now();

        Self {
            id: SafeHouseId::new(),
            operator,
            name: name.into(),
            region,
            capabilities: Vec::new(),
            capacity,
            current_occupancy: 0,
            status: SafeHouseStatus::Available,
            accommodations: Vec::new(),
            max_stay_days: None,
            notes: None,
            verified: false,
            registered_at: now,
            updated_at: now,
        }
    }

    /// Check if this safe house has capacity
    pub fn has_capacity(&self, num_people: u32) -> bool {
        self.current_occupancy + num_people <= self.capacity
    }

    /// Check if this safe house is available
    pub fn is_available(&self) -> bool {
        matches!(
            self.status,
            SafeHouseStatus::Available | SafeHouseStatus::AvailableWithNotice
        ) && self.has_capacity(1)
    }

    /// Check if this safe house can meet specific needs
    pub fn meets_needs(&self, needs: &[SafeHouseCapability]) -> bool {
        needs.iter().all(|need| self.capabilities.contains(need))
    }

    /// Check if this safe house has specific accommodation
    pub fn has_accommodation(&self, accommodation: Accommodation) -> bool {
        self.accommodations.contains(&accommodation)
    }

    /// Add a capability
    pub fn add_capability(&mut self, capability: SafeHouseCapability) {
        if !self.capabilities.contains(&capability) {
            self.capabilities.push(capability);
            self.updated_at = CoarseTimestamp::now();
        }
    }

    /// Add an accommodation
    pub fn add_accommodation(&mut self, accommodation: Accommodation) {
        if !self.accommodations.contains(&accommodation) {
            self.accommodations.push(accommodation);
            self.updated_at = CoarseTimestamp::now();
        }
    }

    /// Update occupancy
    pub fn set_occupancy(&mut self, occupancy: u32) {
        self.current_occupancy = occupancy;
        self.updated_at = CoarseTimestamp::now();

        // Auto-update status based on occupancy
        if self.current_occupancy >= self.capacity {
            self.status = SafeHouseStatus::Occupied;
        } else if self.status == SafeHouseStatus::Occupied {
            self.status = SafeHouseStatus::Available;
        }
    }

    /// Mark as verified (after in-person check)
    pub fn verify(&mut self) {
        self.verified = true;
        self.updated_at = CoarseTimestamp::now();
    }

    /// Update status
    pub fn set_status(&mut self, status: SafeHouseStatus) {
        self.status = status;
        self.updated_at = CoarseTimestamp::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_house_creation() {
        let operator = PersonId::new();
        let region = Region::new("Test Area");
        let house = SafeHouse::new(operator, "Green House", region, 4);

        assert_eq!(house.capacity, 4);
        assert_eq!(house.current_occupancy, 0);
        assert!(house.is_available());
        assert!(house.has_capacity(4));
    }

    #[test]
    fn test_safe_house_occupancy() {
        let mut house = SafeHouse::new(
            PersonId::new(),
            "Test House",
            Region::new("Test"),
            2,
        );

        assert!(house.has_capacity(2));
        assert!(house.is_available());

        house.set_occupancy(2);
        assert!(!house.has_capacity(1));
        assert_eq!(house.status, SafeHouseStatus::Occupied);
        assert!(!house.is_available());
    }

    #[test]
    fn test_safe_house_capabilities() {
        let mut house = SafeHouse::new(
            PersonId::new(),
            "Test House",
            Region::new("Test"),
            4,
        );

        house.add_capability(SafeHouseCapability::Shelter);
        house.add_capability(SafeHouseCapability::Food);
        house.add_capability(SafeHouseCapability::Medical);

        assert!(house.meets_needs(&[
            SafeHouseCapability::Shelter,
            SafeHouseCapability::Food
        ]));

        assert!(!house.meets_needs(&[
            SafeHouseCapability::Shelter,
            SafeHouseCapability::Transportation
        ]));
    }

    #[test]
    fn test_safe_house_accommodations() {
        let mut house = SafeHouse::new(
            PersonId::new(),
            "Test House",
            Region::new("Test"),
            4,
        );

        house.add_accommodation(Accommodation::WheelchairAccessible);
        house.add_accommodation(Accommodation::FamilyFriendly);

        assert!(house.has_accommodation(Accommodation::WheelchairAccessible));
        assert!(house.has_accommodation(Accommodation::FamilyFriendly));
        assert!(!house.has_accommodation(Accommodation::PetFriendly));
    }
}
