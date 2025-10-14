//! Assistance coordination - the heart of the Underground Railroad
//!
//! This module handles:
//! - Emergency requests (immediate help needed)
//! - Safe house registry and matching
//! - Transportation coordination
//! - Supply distribution
//! - Intelligence reports (danger areas, safe routes)

pub mod emergency;
pub mod safe_house;
pub mod transportation;
pub mod intelligence;

pub use emergency::{EmergencyRequest, EmergencyResponse, EmergencyStatus, EmergencyNeed};
pub use safe_house::{SafeHouse, SafeHouseCapability, SafeHouseStatus, Accommodation};
pub use transportation::{TransportOffer, TransportRequest, TransportStatus};
pub use intelligence::{IntelligenceReport, IntelligenceCategory, DangerLevel};
