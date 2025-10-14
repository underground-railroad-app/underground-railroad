//! Veilid network client integration
//!
//! Veilid provides:
//! - Anonymous networking (Tor + I2P + Veilid routing)
//! - Private routes (multi-hop anonymous paths)
//! - DHT (distributed hash table for coordination)
//! - No IP address exposure
//! - Multiple anonymity networks simultaneously

pub mod client;
pub mod routes;
pub mod dht;
pub mod config;

pub use client::{VeilidClient, VeilidState};
pub use routes::{PrivateRoute, RouteId};
pub use dht::{DHTRecord, DHTKey, DHTOperations};
pub use config::VeilidConfig;

use crate::Result;

// Re-export commonly used Veilid types
pub use veilid_core::{
    VeilidAPI,
    VeilidUpdate,
    RoutingContext,
    DHTRecordDescriptor,
    DHTSchema,
    TypedRecordKey,
    CryptoKind,
    CRYPTO_KIND_VLD0,
};
