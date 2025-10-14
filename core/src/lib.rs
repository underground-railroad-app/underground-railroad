//! Underground Railroad Core Library
//!
//! Secure, anonymous coordination network for people fleeing persecution.
//!
//! # Mission
//!
//! Coordinate real-world assistance (safe houses, transportation, supplies,
//! emergency extraction) for people at risk of political persecution.
//!
//! # Security Model
//!
//! - **Threat**: Nation-state adversaries with network surveillance and device seizure
//! - **Defense**: Hardware-backed encryption, Veilid anonymity, post-quantum crypto
//! - **Goal**: Unbreakable security that enables life-saving coordination

pub mod assistance;
pub mod crypto;
pub mod identity;
pub mod messaging;
pub mod storage;
pub mod trust;
pub mod veilid_client;

pub mod types;
pub use types::*;

pub mod error;
pub use error::{Error, Result};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application identifier for Veilid
pub const APP_ID: &str = "underground-railroad";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_app_id() {
        assert_eq!(APP_ID, "underground-railroad");
    }
}
