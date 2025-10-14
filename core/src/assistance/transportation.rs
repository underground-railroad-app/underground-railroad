//! Transportation coordination - helping people move from danger to safety

use crate::{CoarseTimestamp, PersonId, Region, SecureBytes, TransportId};
use serde::{Deserialize, Serialize};

/// A transportation offer - someone willing to drive/help move people
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportOffer {
    /// Unique ID for this offer
    pub id: TransportId,

    /// Who is offering transport
    pub driver: PersonId,

    /// What type of vehicle
    pub vehicle_type: VehicleType,

    /// How many passengers can fit?
    pub capacity: u32,

    /// Regions they can travel from/to
    pub service_regions: Vec<Region>,

    /// Maximum distance willing to drive (km)
    pub max_distance_km: Option<u32>,

    /// Special capabilities
    pub capabilities: Vec<TransportCapability>,

    /// When are they available?
    pub availability: Availability,

    /// Current status
    pub status: TransportStatus,

    /// Additional notes (encrypted)
    pub notes: Option<SecureBytes>,

    /// When was this offer created?
    pub created_at: CoarseTimestamp,

    /// When does this offer expire?
    pub expires_at: CoarseTimestamp,
}

/// A transportation request - someone needs a ride
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportRequest {
    /// Unique ID for this request
    pub id: TransportId,

    /// Who needs transport
    pub requester: PersonId,

    /// From where (approximate)
    pub from_region: Region,

    /// To where (approximate)
    pub to_region: Region,

    /// How many people need transport?
    pub num_people: u32,

    /// How many children?
    pub num_children: u32,

    /// When do they need to travel?
    pub when: TravelTiming,

    /// Special requirements
    pub requirements: Vec<TransportRequirement>,

    /// Additional context (encrypted)
    pub notes: Option<SecureBytes>,

    /// Current status
    pub status: TransportStatus,

    /// When was this request created?
    pub created_at: CoarseTimestamp,

    /// When does this expire?
    pub expires_at: CoarseTimestamp,
}

/// Type of vehicle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VehicleType {
    /// Car/sedan
    Car,

    /// SUV
    Suv,

    /// Van/minivan
    Van,

    /// Truck
    Truck,

    /// Bus/large vehicle
    Bus,

    /// Motorcycle
    Motorcycle,

    /// Bicycle
    Bicycle,

    /// Boat
    Boat,

    /// Other
    Other,
}

/// Special transport capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransportCapability {
    /// Wheelchair accessible
    WheelchairAccessible,

    /// Child car seats available
    ChildSeats,

    /// Can transport luggage/belongings
    LargeStorage,

    /// Discreet/unmarked vehicle
    Discreet,

    /// Knows safe routes/back roads
    SafeRoutes,

    /// Can do long distances (100+ km)
    LongDistance,

    /// Available for immediate pickup
    Immediate,

    /// Can provide food/water
    Provisions,

    /// Knows multiple languages
    Multilingual,

    /// Pet-friendly
    PetFriendly,
}

/// Transport requirements (from requester perspective)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransportRequirement {
    /// Need wheelchair accessibility
    WheelchairAccess,

    /// Traveling with children (need car seats)
    ChildSafety,

    /// Have luggage/belongings
    Luggage,

    /// Need discreet transport
    Discreet,

    /// Avoid main roads/checkpoints
    AvoidCheckpoints,

    /// Long distance travel
    LongDistance,

    /// Need immediate pickup
    Urgent,

    /// Traveling with pets
    Pets,
}

/// When someone needs transport
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TravelTiming {
    /// Need transport immediately
    Immediate,

    /// Within next few hours
    WithinHours,

    /// Within next day
    WithinDay,

    /// Within next few days
    WithinWeek,

    /// Flexible timing
    Flexible,
}

/// Availability of transport
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Availability {
    /// Available right now
    Now,

    /// Available today
    Today,

    /// Available this week
    ThisWeek,

    /// Available with advance notice
    WithNotice,

    /// Not currently available
    Unavailable,
}

/// Status of transport offer/request
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransportStatus {
    /// Active - still looking
    Active,

    /// Matched - driver and passenger connected
    Matched,

    /// In progress - currently transporting
    InProgress,

    /// Completed successfully
    Completed,

    /// Cancelled
    Cancelled,

    /// Expired
    Expired,
}

impl TransportOffer {
    /// Create a new transport offer
    pub fn new(
        driver: PersonId,
        vehicle_type: VehicleType,
        capacity: u32,
        service_regions: Vec<Region>,
    ) -> Self {
        let now = CoarseTimestamp::now();
        let expires_at = CoarseTimestamp::from_datetime(
            chrono::Utc::now() + chrono::Duration::days(7)
        );

        Self {
            id: TransportId::new(),
            driver,
            vehicle_type,
            capacity,
            service_regions,
            max_distance_km: None,
            capabilities: Vec::new(),
            availability: Availability::WithNotice,
            status: TransportStatus::Active,
            notes: None,
            created_at: now,
            expires_at,
        }
    }

    /// Check if this offer can serve a request
    pub fn can_serve(&self, request: &TransportRequest) -> bool {
        // Must be active
        if self.status != TransportStatus::Active {
            return false;
        }

        // Must have capacity
        if self.capacity < request.num_people {
            return false;
        }

        // Check if service regions overlap with request
        let serves_from = self.service_regions.iter().any(|r| {
            request.from_region.distance_to(r).map_or(false, |d| d < 50.0)
        });

        let serves_to = self.service_regions.iter().any(|r| {
            request.to_region.distance_to(r).map_or(false, |d| d < 50.0)
        });

        if !serves_from && !serves_to {
            return false;
        }

        // Check capabilities vs requirements
        for req in &request.requirements {
            let capability = match req {
                TransportRequirement::WheelchairAccess => TransportCapability::WheelchairAccessible,
                TransportRequirement::ChildSafety => TransportCapability::ChildSeats,
                TransportRequirement::Luggage => TransportCapability::LargeStorage,
                TransportRequirement::Discreet => TransportCapability::Discreet,
                TransportRequirement::AvoidCheckpoints => TransportCapability::SafeRoutes,
                TransportRequirement::LongDistance => TransportCapability::LongDistance,
                TransportRequirement::Urgent => TransportCapability::Immediate,
                TransportRequirement::Pets => TransportCapability::PetFriendly,
            };

            if !self.capabilities.contains(&capability) {
                return false;
            }
        }

        true
    }

    /// Add a capability
    pub fn add_capability(&mut self, capability: TransportCapability) {
        if !self.capabilities.contains(&capability) {
            self.capabilities.push(capability);
        }
    }
}

impl TransportRequest {
    /// Create a new transport request
    pub fn new(
        requester: PersonId,
        from_region: Region,
        to_region: Region,
        num_people: u32,
        when: TravelTiming,
    ) -> Self {
        let now = CoarseTimestamp::now();
        let expires_at = CoarseTimestamp::from_datetime(
            chrono::Utc::now() + chrono::Duration::days(3)
        );

        Self {
            id: TransportId::new(),
            requester,
            from_region,
            to_region,
            num_people,
            num_children: 0,
            when,
            requirements: Vec::new(),
            notes: None,
            status: TransportStatus::Active,
            created_at: now,
            expires_at,
        }
    }

    /// Check if this request is still active
    pub fn is_active(&self) -> bool {
        self.status == TransportStatus::Active &&
        !self.expires_at.is_expired(chrono::Duration::zero())
    }

    /// Add a requirement
    pub fn add_requirement(&mut self, requirement: TransportRequirement) {
        if !self.requirements.contains(&requirement) {
            self.requirements.push(requirement);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_offer() {
        let driver = PersonId::new();
        let region = Region::new("Test City");

        let mut offer = TransportOffer::new(
            driver,
            VehicleType::Van,
            5,
            vec![region],
        );

        offer.add_capability(TransportCapability::WheelchairAccessible);
        offer.add_capability(TransportCapability::ChildSeats);

        assert_eq!(offer.capacity, 5);
        assert_eq!(offer.vehicle_type, VehicleType::Van);
        assert!(offer.capabilities.contains(&TransportCapability::WheelchairAccessible));
    }

    #[test]
    fn test_transport_matching() {
        let driver = PersonId::new();
        let requester = PersonId::new();
        let from = Region::new("City A");
        let to = Region::new("City B");

        let mut offer = TransportOffer::new(
            driver,
            VehicleType::Van,
            5,
            vec![from.clone(), to.clone()],
        );
        offer.add_capability(TransportCapability::ChildSeats);

        let mut request = TransportRequest::new(
            requester,
            from,
            to,
            3,
            TravelTiming::WithinDay,
        );
        request.add_requirement(TransportRequirement::ChildSafety);

        assert!(offer.can_serve(&request));
    }

    #[test]
    fn test_transport_mismatch_capacity() {
        let driver = PersonId::new();
        let requester = PersonId::new();
        let region = Region::new("Test");

        let offer = TransportOffer::new(
            driver,
            VehicleType::Car,
            2,  // Only 2 seats
            vec![region.clone()],
        );

        let request = TransportRequest::new(
            requester,
            region.clone(),
            region,
            5,  // Need 5 seats
            TravelTiming::Immediate,
        );

        assert!(!offer.can_serve(&request));
    }
}
