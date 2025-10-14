//! Veilid DHT operations for distributed coordination
//!
//! The DHT (Distributed Hash Table) allows nodes to:
//! - Share data without central server
//! - Coordinate asynchronously (offline message delivery)
//! - Discover other nodes in the network
//! - Store encrypted announcements (safe houses, transport offers, etc.)

use crate::{PersonId, Result};
use serde::{Deserialize, Serialize};
use veilid_core::{VeilidAPI, DHTSchema, DHTRecordDescriptor, CRYPTO_KIND_VLD0, TypedRecordKey};

/// DHT key (256-bit identifier)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DHTKey(pub [u8; 32]);

impl DHTKey {
    /// Create from bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Generate random DHT key
    pub fn random() -> Self {
        use rand::RngCore;
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        Self(bytes)
    }

    /// Get as bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Parse from hex string
    pub fn from_hex(hex: &str) -> Result<Self> {
        let bytes = hex::decode(hex)
            .map_err(|e| crate::Error::Serialization(format!("Hex decode failed: {}", e)))?;

        if bytes.len() != 32 {
            return Err(crate::Error::Invalid("DHT key must be 32 bytes".to_string()));
        }

        let mut array = [0u8; 32];
        array.copy_from_slice(&bytes);

        Ok(Self(array))
    }
}

/// A DHT record (encrypted data stored in DHT)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DHTRecord {
    /// DHT key where this is stored
    pub key: DHTKey,

    /// Encrypted data
    pub data: Vec<u8>,

    /// When was this written?
    pub written_at: crate::CoarseTimestamp,

    /// Is this a private record? (only accessible to key holder)
    pub private: bool,
}

impl DHTRecord {
    /// Create a new DHT record
    pub fn new(key: DHTKey, data: Vec<u8>, private: bool) -> Self {
        Self {
            key,
            data,
            written_at: crate::CoarseTimestamp::now(),
            private,
        }
    }

    /// Get data
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get key
    pub fn key(&self) -> &DHTKey {
        &self.key
    }

    /// Check if this is a private record
    pub fn is_private(&self) -> bool {
        self.private
    }
}

/// DHT operations wrapper
pub struct DHTOperations {
    /// Reference to Veilid API
    api: VeilidAPI,
}

impl DHTOperations {
    /// Create new DHT operations with VeilidAPI
    pub fn new(api: VeilidAPI) -> Self {
        Self { api }
    }

    /// Write data to DHT (public)
    ///
    /// Creates a new public DHT record (key provided by Veilid)
    pub async fn write_public(&self, data: &[u8]) -> Result<(DHTKey, DHTRecordDescriptor)> {
        // Create routing context with privacy
        let routing_context = self.api.routing_context()?
            .with_default_safety()?;

        // Create a new public DHT record
        // Use a simple DFLT schema with one subkey
        let schema = DHTSchema::dflt(1)?;
        let record_descriptor = routing_context.create_dht_record(schema, None, None).await?;

        // Write data to subkey 0
        routing_context.set_dht_value(
            record_descriptor.key().clone(),
            0, // subkey
            data.to_vec(),
            None, // No writer (owner writes)
        ).await?;

        // Extract the key bytes
        let key_bytes = record_descriptor.key().value.bytes;
        let mut dht_key = [0u8; 32];
        dht_key.copy_from_slice(&key_bytes[..32]);

        Ok((DHTKey(dht_key), record_descriptor))
    }

    /// Write data to DHT (private - only accessible to key holder)
    ///
    /// Creates or updates a private DHT record with encryption
    pub async fn write_private(&self, data: &[u8]) -> Result<(DHTKey, DHTRecordDescriptor)> {
        // Create routing context with privacy
        let routing_context = self.api.routing_context()?
            .with_default_safety()?;

        // Create a private DHT record with encryption
        // SMPL schema means the record is member-only (encrypted for specific keys)
        let schema = DHTSchema::smpl(
            1, // member count
            vec![], // no specific members yet (will use owner)
        )?;

        let record_descriptor = routing_context.create_dht_record(schema, None, None).await?;

        // Write encrypted data to subkey 0
        routing_context.set_dht_value(
            record_descriptor.key().clone(),
            0, // subkey
            data.to_vec(),
            None, // Owner writes
        ).await?;

        // Convert TypedKey to our DHTKey
        let key_bytes = record_descriptor.key().value.bytes;
        let mut dht_key = [0u8; 32];
        dht_key.copy_from_slice(&key_bytes[..32]);

        Ok((DHTKey(dht_key), record_descriptor))
    }

    /// Read data from DHT using a DHTRecordDescriptor
    ///
    /// Note: Reading by raw key is complex; typically you store the DHTRecordDescriptor
    pub async fn read(&self, record_descriptor: &DHTRecordDescriptor) -> Result<Option<Vec<u8>>> {
        // Create routing context with privacy
        let routing_context = self.api.routing_context()?
            .with_default_safety()?;

        // Read data from subkey 0
        let value_data = routing_context.get_dht_value(
            record_descriptor.key().clone(),
            0, // subkey
            false, // Don't force refresh
        ).await?;

        Ok(value_data.map(|vd| vd.data().to_vec()))
    }

    /// Delete data from DHT
    ///
    /// Note: DHT deletion is not guaranteed (data may persist on other nodes)
    pub async fn delete(&self, record_descriptor: &DHTRecordDescriptor) -> Result<()> {
        // Create routing context with privacy
        let routing_context = self.api.routing_context()?
            .with_default_safety()?;

        // Delete the record
        routing_context.delete_dht_record(record_descriptor.key().clone()).await?;

        Ok(())
    }

    /// Create a mailbox for offline messages
    ///
    /// Returns a DHT key where others can leave messages
    pub async fn create_mailbox(&self, _persona: PersonId) -> Result<(DHTKey, DHTRecordDescriptor)> {
        // Create routing context with privacy
        let routing_context = self.api.routing_context()?
            .with_default_safety()?;

        // Create a private DHT record for mailbox
        // Use SMPL schema to allow multiple members to write
        let schema = DHTSchema::smpl(
            10, // Allow up to 10 different writers
            vec![], // No specific members initially
        )?;

        let record_descriptor = routing_context.create_dht_record(schema, None, None).await?;

        // Convert TypedKey to our DHTKey
        let key_bytes = record_descriptor.key().value.bytes;
        let mut dht_key = [0u8; 32];
        dht_key.copy_from_slice(&key_bytes[..32]);

        tracing::info!("Created mailbox for persona with key: {}", hex::encode(&dht_key));

        Ok((DHTKey(dht_key), record_descriptor))
    }

    /// Check mailbox for new messages
    ///
    /// Reads all subkeys from the mailbox DHT record
    pub async fn check_mailbox(&self, mailbox_descriptor: &DHTRecordDescriptor) -> Result<Vec<Vec<u8>>> {
        // Create routing context with privacy
        let routing_context = self.api.routing_context()?
            .with_default_safety()?;

        let mut messages = Vec::new();

        // Read from multiple subkeys (messages are stored in different subkeys)
        // Try reading first 10 subkeys
        for subkey in 0..10 {
            match routing_context.get_dht_value(
                mailbox_descriptor.key().clone(),
                subkey,
                false, // Don't force refresh
            ).await {
                Ok(Some(value_data)) => {
                    messages.push(value_data.data().to_vec());
                }
                Ok(None) => {
                    // No more messages in this subkey
                }
                Err(_) => {
                    // Error reading subkey, skip
                    break;
                }
            }
        }

        Ok(messages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::veilid_client::{PrivateRoute, RouteId};
    use crate::veilid_client::routes::RouteManager;

    #[test]
    fn test_dht_key_random() {
        let key1 = DHTKey::random();
        let key2 = DHTKey::random();

        // Should be different
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_dht_key_hex() {
        let key = DHTKey::from_bytes([42u8; 32]);
        let hex = key.to_hex();

        let parsed = DHTKey::from_hex(&hex).unwrap();
        assert_eq!(parsed, key);
    }

    #[test]
    fn test_dht_key_invalid_hex() {
        let result = DHTKey::from_hex("invalid");
        assert!(result.is_err());

        // Too short
        let result = DHTKey::from_hex("0123");
        assert!(result.is_err());
    }

    #[test]
    fn test_dht_record() {
        let key = DHTKey::random();
        let data = vec![1, 2, 3, 4];

        let record = DHTRecord::new(key.clone(), data.clone(), true);

        assert_eq!(record.key(), &key);
        assert_eq!(record.data(), &data);
        assert!(record.is_private());
    }

    #[test]
    fn test_private_route() {
        let id = RouteId("test-route-123".to_string());
        let blob = vec![1, 2, 3, 4, 5, 6, 7, 8];

        let route = PrivateRoute::new(id.clone(), blob);

        assert_eq!(route.id, id);
        assert!(route.is_valid());
    }

    #[test]
    fn test_route_export_import() {
        let id = RouteId("test".to_string());
        let blob = vec![1, 2, 3, 4, 5];
        let route = PrivateRoute::new(id.clone(), blob.clone());

        let exported = route.export().unwrap();
        let imported = PrivateRoute::import(id, &exported).unwrap();

        assert_eq!(imported.route_blob(), &blob);
    }

    #[test]
    fn test_route_manager() {
        let mut manager = RouteManager::new();

        let persona1 = PersonId::new();
        let persona2 = PersonId::new();

        let route1 = PrivateRoute::new(RouteId("r1".to_string()), vec![1]);
        let route2 = PrivateRoute::new(RouteId("r2".to_string()), vec![2]);

        manager.add_route(persona1, route1);
        manager.add_route(persona2, route2);

        assert_eq!(manager.count(), 2);

        let routes = manager.list_routes();
        assert_eq!(routes.len(), 2);
    }

    #[test]
    fn test_route_manager_invalidate_all() {
        let mut manager = RouteManager::new();

        let persona = PersonId::new();
        let route = PrivateRoute::new(RouteId("test".to_string()), vec![1]);

        manager.add_route(persona, route);
        assert!(manager.get_route(persona).unwrap().is_valid());

        manager.invalidate_all();
        assert!(!manager.get_route(persona).unwrap().is_valid());
    }
}
