//! Core type definitions for the Underground Railroad

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Unique identifier for a person in the network
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PersonId(pub Uuid);

impl PersonId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for PersonId {
    fn default() -> Self {
        Self::new()
    }
}

/// Unique identifier for a safe house
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SafeHouseId(pub Uuid);

impl SafeHouseId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for SafeHouseId {
    fn default() -> Self {
        Self::new()
    }
}

/// Unique identifier for a transportation offer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TransportId(pub Uuid);

impl TransportId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for TransportId {
    fn default() -> Self {
        Self::new()
    }
}

/// Unique identifier for an emergency request
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EmergencyId(pub Uuid);

impl EmergencyId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for EmergencyId {
    fn default() -> Self {
        Self::new()
    }
}

/// Geographic region (coarse, not exact location)
///
/// For privacy, we never store exact addresses. Regions are approximate
/// areas like "Northeast Seattle" or "Downtown Boston".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Region {
    /// Human-readable name (e.g., "Northeast Seattle")
    pub name: String,

    /// Approximate center latitude (rounded to ~1km)
    pub lat: Option<i32>,  // Stored as millionths of degrees, rounded

    /// Approximate center longitude (rounded to ~1km)
    pub lon: Option<i32>,  // Stored as millionths of degrees, rounded

    /// Approximate radius in kilometers
    pub radius_km: Option<u32>,
}

impl Region {
    /// Create a new region with just a name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            lat: None,
            lon: None,
            radius_km: None,
        }
    }

    /// Estimate distance to another region (very approximate)
    pub fn distance_to(&self, other: &Region) -> Option<f64> {
        let (lat1, lon1) = (self.lat?, self.lon?);
        let (lat2, lon2) = (other.lat?, other.lon?);

        // Simple Euclidean distance (good enough for coarse regions)
        let lat_diff = (lat1 - lat2) as f64 / 1_000_000.0;
        let lon_diff = (lon1 - lon2) as f64 / 1_000_000.0;

        // Rough conversion: 1 degree ~ 111km
        let km = ((lat_diff * lat_diff + lon_diff * lon_diff).sqrt()) * 111.0;
        Some(km)
    }
}

/// Trust level for a contact
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum TrustLevel {
    /// Blocked - do not trust
    Blocked = 0,

    /// Unknown - never verified
    Unknown = 1,

    /// Introduced by someone trusted (friend-of-friend)
    Introduced = 2,

    /// Verified remotely (video call, encrypted verification)
    VerifiedRemote = 3,

    /// Verified in person (highest trust)
    VerifiedInPerson = 4,
}

impl TrustLevel {
    /// Can this person see my requests/offers?
    pub fn can_see_activity(&self) -> bool {
        matches!(
            self,
            TrustLevel::VerifiedInPerson | TrustLevel::VerifiedRemote | TrustLevel::Introduced
        )
    }

    /// Can this person relay messages for me?
    pub fn can_relay(&self) -> bool {
        matches!(self, TrustLevel::VerifiedInPerson | TrustLevel::VerifiedRemote)
    }
}

/// Urgency level for requests
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Urgency {
    /// Low - planning ahead
    Low = 1,

    /// Medium - within days
    Medium = 2,

    /// High - within hours
    High = 3,

    /// Critical - immediate danger (minutes)
    Critical = 4,
}

impl Urgency {
    /// How many hops should this propagate through the network?
    pub fn propagation_hops(&self) -> u8 {
        match self {
            Urgency::Low => 1,
            Urgency::Medium => 2,
            Urgency::High => 3,
            Urgency::Critical => 5,
        }
    }

    /// How long until this request expires?
    pub fn default_expiry(&self) -> chrono::Duration {
        match self {
            Urgency::Low => chrono::Duration::days(7),
            Urgency::Medium => chrono::Duration::days(3),
            Urgency::High => chrono::Duration::hours(24),
            Urgency::Critical => chrono::Duration::hours(6),
        }
    }
}

/// Timestamp with coarse precision (prevents timing attacks)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CoarseTimestamp(i64);

impl CoarseTimestamp {
    /// Create timestamp rounded to nearest 5 minutes
    pub fn now() -> Self {
        let now = Utc::now().timestamp();
        let rounded = (now / 300) * 300; // Round to 5-minute intervals
        Self(rounded)
    }

    /// Create from a specific time (will be rounded)
    pub fn from_datetime(dt: DateTime<Utc>) -> Self {
        let ts = dt.timestamp();
        let rounded = (ts / 300) * 300;
        Self(rounded)
    }

    /// Get as Unix timestamp (seconds since epoch)
    pub fn as_secs(&self) -> i64 {
        self.0
    }

    /// Check if this timestamp has expired
    pub fn is_expired(&self, ttl: chrono::Duration) -> bool {
        let now = Utc::now().timestamp();
        now > self.0 + ttl.num_seconds()
    }
}

/// Cryptographic fingerprint for verification
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fingerprint([u8; 32]);

impl Fingerprint {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Convert to human-readable words for verification
    /// Uses a subset of BIP39 wordlist for easy verification
    pub fn to_words(&self) -> [&'static str; 3] {
        // Use first 12 bytes, split into 3 words
        let idx1 = u16::from_be_bytes([self.0[0], self.0[1]]) as usize % VERIFICATION_WORDS.len();
        let idx2 = u16::from_be_bytes([self.0[2], self.0[3]]) as usize % VERIFICATION_WORDS.len();
        let idx3 = u16::from_be_bytes([self.0[4], self.0[5]]) as usize % VERIFICATION_WORDS.len();

        [
            VERIFICATION_WORDS[idx1],
            VERIFICATION_WORDS[idx2],
            VERIFICATION_WORDS[idx3],
        ]
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
}

impl std::fmt::Debug for Fingerprint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Fingerprint({}...)", &hex::encode(&self.0[..4]))
    }
}

impl std::fmt::Display for Fingerprint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &hex::encode(&self.0[..8]))
    }
}

// Subset of easy-to-say words for fingerprint verification
const VERIFICATION_WORDS: &[&str] = &[
    "dolphin", "mountain", "coffee", "ocean", "butterfly", "galaxy",
    "thunder", "crystal", "rainbow", "forest", "river", "sunset",
    "meadow", "canyon", "lighthouse", "willow", "compass", "anchor",
    "maple", "phoenix", "aurora", "falcon", "jasmine", "coral",
];

/// Secure bytes that are zeroized on drop
#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct SecureBytes(Vec<u8>);

impl SecureBytes {
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl std::fmt::Debug for SecureBytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SecureBytes([REDACTED {} bytes])", self.0.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_person_id() {
        let id1 = PersonId::new();
        let id2 = PersonId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_region_distance() {
        let seattle = Region {
            name: "Seattle".to_string(),
            lat: Some(47_606_000),  // 47.606°N
            lon: Some(-122_332_000), // -122.332°W
            radius_km: Some(10),
        };

        let portland = Region {
            name: "Portland".to_string(),
            lat: Some(45_505_000),  // 45.505°N
            lon: Some(-122_675_000), // -122.675°W
            radius_km: Some(10),
        };

        let distance = seattle.distance_to(&portland);
        assert!(distance.is_some());
        // Seattle to Portland is ~233km
        let dist = distance.unwrap();
        assert!(dist > 200.0 && dist < 300.0);
    }

    #[test]
    fn test_trust_level_ordering() {
        assert!(TrustLevel::VerifiedInPerson > TrustLevel::VerifiedRemote);
        assert!(TrustLevel::VerifiedRemote > TrustLevel::Introduced);
        assert!(TrustLevel::Introduced > TrustLevel::Unknown);
        assert!(TrustLevel::Unknown > TrustLevel::Blocked);
    }

    #[test]
    fn test_coarse_timestamp() {
        let ts1 = CoarseTimestamp::now();
        let ts2 = CoarseTimestamp::now();
        // Should be same or very close (5min intervals)
        assert_eq!(ts1, ts2);
    }

    #[test]
    fn test_fingerprint_words() {
        let fp = Fingerprint::new([0u8; 32]);
        let words = fp.to_words();
        assert_eq!(words.len(), 3);
        assert!(VERIFICATION_WORDS.contains(&words[0]));
    }

    #[test]
    fn test_urgency_propagation() {
        assert_eq!(Urgency::Critical.propagation_hops(), 5);
        assert_eq!(Urgency::High.propagation_hops(), 3);
        assert_eq!(Urgency::Low.propagation_hops(), 1);
    }
}
