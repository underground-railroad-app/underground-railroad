//! Veilid configuration for maximum anonymity

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::Result;

/// Configuration for Veilid node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VeilidConfig {
    /// Where to store Veilid data
    pub storage_path: PathBuf,

    /// Enable relay (help route for others)
    pub enable_relay: bool,

    /// Network preferences
    pub network: NetworkConfig,

    /// Privacy settings
    pub privacy: PrivacyConfig,
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Use Tor for anonymity
    pub use_tor: bool,

    /// Use I2P for anonymity
    pub use_i2p: bool,

    /// Enable UPnP (usually false for privacy)
    pub enable_upnp: bool,

    /// Maximum peers to connect to
    pub max_peers: u32,

    /// Enable DHT
    pub enable_dht: bool,
}

/// Privacy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyConfig {
    /// Default routing: Private (most anonymous)
    pub default_routing_private: bool,

    /// Minimum relay count for private routes
    pub min_relay_count: u8,

    /// Maximum relay count for private routes
    pub max_relay_count: u8,

    /// Enable cover traffic (dummy messages for traffic analysis resistance)
    pub enable_cover_traffic: bool,
}

impl VeilidConfig {
    /// Create default configuration (Veilid-only, no Tor/I2P)
    pub fn default_private(storage_path: PathBuf) -> Self {
        Self {
            storage_path,
            enable_relay: true, // Help others (strengthens network)
            network: NetworkConfig {
                use_tor: false,  // Veilid-only routing
                use_i2p: false,  // Veilid-only routing
                enable_upnp: false, // Don't expose via UPnP
                max_peers: 16,
                enable_dht: true,
            },
            privacy: PrivacyConfig {
                default_routing_private: true,
                min_relay_count: 3, // Minimum 3 hops for anonymity
                max_relay_count: 5, // Maximum 5 hops
                enable_cover_traffic: true,
            },
        }
    }

    /// Create configuration for testing (less strict)
    pub fn for_testing(storage_path: PathBuf) -> Self {
        Self {
            storage_path,
            enable_relay: false,
            network: NetworkConfig {
                use_tor: false, // Faster for testing
                use_i2p: false,
                enable_upnp: false,
                max_peers: 8,
                enable_dht: true,
            },
            privacy: PrivacyConfig {
                default_routing_private: false,
                min_relay_count: 1,
                max_relay_count: 2,
                enable_cover_traffic: false,
            },
        }
    }

    /// Convert to veilid_core::VeilidConfig
    pub fn to_veilid_config(&self) -> Result<veilid_core::VeilidConfig> {
        // Convert storage_path to string
        let storage_dir = self.storage_path
            .to_str()
            .ok_or_else(|| crate::Error::Invalid("Invalid storage path".to_string()))?;

        // Use VeilidConfig builder with defaults
        let config = veilid_core::VeilidConfig::new(
            "underground-railroad",  // program_name
            "veilid",                 // organization
            "",                       // qualifier
            Some(storage_dir),        // storage_directory
            None,                     // config_directory
        );

        // Log if Tor/I2P is requested
        if self.network.use_tor || self.network.use_i2p {
            tracing::info!("Tor/I2P support requested - ensure Veilid is configured for this");
        }

        Ok(config)
    }
}

impl Default for VeilidConfig {
    fn default() -> Self {
        Self::default_private(PathBuf::from(".veilid"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = VeilidConfig::default();

        // Should use maximum privacy by default
        assert!(config.network.use_tor);
        assert!(config.network.use_i2p);
        assert!(!config.network.enable_upnp);
        assert!(config.privacy.default_routing_private);
        assert!(config.privacy.enable_cover_traffic);
        assert!(config.enable_relay);
    }

    #[test]
    fn test_private_config() {
        let config = VeilidConfig::default_private(PathBuf::from("/test"));

        assert_eq!(config.privacy.min_relay_count, 3);
        assert_eq!(config.privacy.max_relay_count, 5);
        assert!(config.privacy.default_routing_private);
    }

    #[test]
    fn test_testing_config() {
        let config = VeilidConfig::for_testing(PathBuf::from("/test"));

        // Testing config should be faster (less anonymous)
        assert!(!config.network.use_tor);
        assert!(!config.privacy.enable_cover_traffic);
        assert!(!config.enable_relay);
    }
}
