//! Veilid-based messaging using DHT mailboxes
//!
//! Each user has a DHT-based mailbox where others can leave encrypted messages.
//! Messages are written as subkeys in a private DHT record.

use crate::{
    messaging::Message,
    veilid_client::VeilidClient,
    PersonId, Result,
};
use veilid_core::DHTRecordDescriptor;

/// Mailbox manager for sending and receiving messages via Veilid DHT
pub struct VeilidMailbox {
    /// Veilid client
    client: VeilidClient,

    /// Our mailbox descriptor (for receiving)
    mailbox_descriptor: Option<DHTRecordDescriptor>,

    /// Next subkey to write to in our mailbox
    next_subkey: u32,
}

impl VeilidMailbox {
    /// Create a new mailbox manager
    pub fn new(client: VeilidClient) -> Self {
        Self {
            client,
            mailbox_descriptor: None,
            next_subkey: 0,
        }
    }

    /// Create a mailbox for this user
    ///
    /// Returns the serialized DHT record descriptor that should be stored in Identity
    pub async fn create_mailbox(&mut self, persona_id: PersonId) -> Result<Vec<u8>> {
        if !self.client.is_connected() {
            return Err(crate::Error::Network("Veilid not connected".to_string()));
        }

        let api = self.client.api()
            .ok_or_else(|| crate::Error::Network("Veilid API not available".to_string()))?;

        // Create routing context with privacy
        let routing_context = api.routing_context()?
            .with_default_safety()?;

        // Create a private DHT record for mailbox
        // Use SMPL schema to allow multiple members to write (senders)
        let schema = veilid_core::DHTSchema::smpl(
            50, // Allow up to 50 different senders
            vec![], // No specific members initially (owner can read all)
        )?;

        let record_descriptor = routing_context.create_dht_record(schema, None, None).await?;

        tracing::info!(
            "Created Veilid mailbox for {:?} with key: {}",
            persona_id,
            hex::encode(record_descriptor.key().value.bytes)
        );

        // Serialize the descriptor for storage
        let serialized = bincode::serialize(&record_descriptor)
            .map_err(|e| crate::Error::Serialization(format!("Failed to serialize mailbox: {}", e)))?;

        self.mailbox_descriptor = Some(record_descriptor);
        self.next_subkey = 0;

        Ok(serialized)
    }

    /// Load mailbox from serialized descriptor
    pub fn load_mailbox(&mut self, serialized: &[u8]) -> Result<()> {
        let descriptor: DHTRecordDescriptor = bincode::deserialize(serialized)
            .map_err(|e| crate::Error::Serialization(format!("Failed to deserialize mailbox: {}", e)))?;

        self.mailbox_descriptor = Some(descriptor);
        self.next_subkey = 0;

        Ok(())
    }

    /// Send a message to a contact's mailbox
    ///
    /// recipient_mailbox_key: serialized DHTRecordDescriptor of recipient's mailbox
    pub async fn send_message(
        &self,
        message: &Message,
        recipient_mailbox_key: &[u8],
    ) -> Result<()> {
        if !self.client.is_connected() {
            return Err(crate::Error::Network("Veilid not connected".to_string()));
        }

        let api = self.client.api()
            .ok_or_else(|| crate::Error::Network("Veilid API not available".to_string()))?;

        // Deserialize recipient's mailbox descriptor
        let recipient_descriptor: DHTRecordDescriptor = bincode::deserialize(recipient_mailbox_key)
            .map_err(|e| crate::Error::Serialization(format!("Invalid mailbox key: {}", e)))?;

        // Serialize the message
        let message_bytes = bincode::serialize(message)
            .map_err(|e| crate::Error::Serialization(format!("Failed to serialize message: {}", e)))?;

        // Create routing context with privacy
        let routing_context = api.routing_context()?
            .with_default_safety()?;

        // Find an empty subkey to write to
        // Try subkeys 0-49 (matching the schema limit)
        let mut written = false;
        for subkey in 0..50 {
            // Check if subkey is empty
            match routing_context.get_dht_value(
                recipient_descriptor.key().clone(),
                subkey,
                false, // Don't force refresh
            ).await {
                Ok(None) => {
                    // Empty subkey, write here
                    routing_context.set_dht_value(
                        recipient_descriptor.key().clone(),
                        subkey,
                        message_bytes.clone(),
                        None, // No specific writer (use our routing context)
                    ).await?;

                    tracing::info!("Sent message to subkey {} of recipient's mailbox", subkey);
                    written = true;
                    break;
                }
                Ok(Some(_)) => {
                    // Subkey occupied, try next
                    continue;
                }
                Err(e) => {
                    // Error reading, might be permission issue or network problem
                    tracing::warn!("Error checking subkey {}: {:?}", subkey, e);
                    continue;
                }
            }
        }

        if !written {
            return Err(crate::Error::Network("Mailbox full - no empty subkeys".to_string()));
        }

        Ok(())
    }

    /// Poll mailbox for new messages
    ///
    /// Returns list of messages found (and removes them from DHT)
    pub async fn poll_messages(&mut self) -> Result<Vec<Message>> {
        if !self.client.is_connected() {
            return Err(crate::Error::Network("Veilid not connected".to_string()));
        }

        let mailbox_descriptor = self.mailbox_descriptor.as_ref()
            .ok_or_else(|| crate::Error::Invalid("No mailbox created or loaded".to_string()))?;

        let api = self.client.api()
            .ok_or_else(|| crate::Error::Network("Veilid API not available".to_string()))?;

        // Create routing context with privacy
        let routing_context = api.routing_context()?
            .with_default_safety()?;

        let mut messages = Vec::new();

        // Check all subkeys for messages
        for subkey in 0..50 {
            match routing_context.get_dht_value(
                mailbox_descriptor.key().clone(),
                subkey,
                true, // Force refresh to get latest
            ).await {
                Ok(Some(value_data)) => {
                    // Found a message
                    let message_bytes = value_data.data();

                    match bincode::deserialize::<Message>(message_bytes) {
                        Ok(message) => {
                            messages.push(message);

                            // Clear the subkey after reading
                            // Note: In Veilid, we can't truly delete, but we can write empty data
                            let _ = routing_context.set_dht_value(
                                mailbox_descriptor.key().clone(),
                                subkey,
                                vec![], // Empty data to mark as "read"
                                None,
                            ).await;

                            tracing::info!("Retrieved message from subkey {}", subkey);
                        }
                        Err(e) => {
                            tracing::warn!("Failed to deserialize message from subkey {}: {:?}", subkey, e);
                        }
                    }
                }
                Ok(None) => {
                    // Empty subkey, no message
                }
                Err(e) => {
                    tracing::warn!("Error reading subkey {}: {:?}", subkey, e);
                }
            }
        }

        tracing::info!("Polled mailbox and found {} new messages", messages.len());

        Ok(messages)
    }

    /// Get mailbox key for sharing with contacts
    ///
    /// Returns serialized DHTRecordDescriptor
    pub fn get_mailbox_key(&self) -> Result<Vec<u8>> {
        let descriptor = self.mailbox_descriptor.as_ref()
            .ok_or_else(|| crate::Error::Invalid("No mailbox created or loaded".to_string()))?;

        bincode::serialize(descriptor)
            .map_err(|e| crate::Error::Serialization(format!("Failed to serialize mailbox key: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::veilid_client::VeilidConfig;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_mailbox_creation() {
        let tmp = TempDir::new().unwrap();
        let config = VeilidConfig::for_testing(tmp.path().to_path_buf());
        let mut client = VeilidClient::new(config);

        // Start client (in test mode, this doesn't actually connect to Veilid)
        client.start().await.unwrap();

        let mut mailbox = VeilidMailbox::new(client);
        let persona_id = PersonId::new();

        // In test mode, this will fail because Veilid API is not actually available
        // In production with real Veilid, this would create an actual DHT mailbox
        let result = mailbox.create_mailbox(persona_id).await;

        // Expected to fail in test mode
        assert!(result.is_err());
    }
}
