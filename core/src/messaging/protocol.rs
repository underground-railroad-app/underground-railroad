//! Message protocol definitions

use crate::{
    PersonId, EmergencyId, SafeHouseId, CoarseTimestamp,
    assistance::{EmergencyRequest, EmergencyResponse, IntelligenceReport},
    SecureBytes,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A message in the Underground Railroad network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique message ID
    pub id: Uuid,

    /// Who sent this message
    pub sender: PersonId,

    /// Who should receive this message
    pub recipient: PersonId,

    /// Type and content of message
    pub message_type: MessageType,

    /// When was this message created?
    pub created_at: CoarseTimestamp,

    /// When does this message expire?
    pub expires_at: Option<CoarseTimestamp>,

    /// Message status
    pub status: MessageStatus,

    /// Number of relay hops (for tracking propagation)
    pub hop_count: u8,
}

/// Types of messages that can be sent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    /// Simple text message
    Text {
        content: String,
    },

    /// Emergency assistance request
    Emergency {
        request: EmergencyRequest,
    },

    /// Response to emergency
    EmergencyResponse {
        emergency_id: EmergencyId,
        response: EmergencyResponse,
    },

    /// Intelligence report
    Intelligence {
        report: IntelligenceReport,
    },

    /// Connection request (want to add someone as contact)
    ConnectionRequest {
        fingerprint: crate::Fingerprint,
        public_key: Vec<u8>,
        introduction: Option<PersonId>,
    },

    /// Connection accepted
    ConnectionAccepted {
        fingerprint: crate::Fingerprint,
        public_key: Vec<u8>,
    },

    /// Connection rejected
    ConnectionRejected {
        reason: Option<String>,
    },

    /// Safe house availability update
    SafeHouseUpdate {
        safe_house_id: SafeHouseId,
        available: bool,
        capacity: u32,
    },

    /// Relay request (forward message to someone else)
    Relay {
        target: PersonId,
        encrypted_payload: Vec<u8>,
    },

    /// Read receipt
    ReadReceipt {
        message_id: Uuid,
        read_at: CoarseTimestamp,
    },

    /// Delivery confirmation
    DeliveryConfirmation {
        message_id: Uuid,
        delivered_at: CoarseTimestamp,
    },
}

/// Message delivery status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageStatus {
    /// Draft - not sent yet
    Draft,

    /// Queued for sending (offline)
    Queued,

    /// Sending in progress
    Sending,

    /// Sent successfully
    Sent,

    /// Delivered to recipient
    Delivered,

    /// Read by recipient
    Read,

    /// Failed to send
    Failed,

    /// Expired
    Expired,
}

/// Encrypted message envelope (what goes over the wire)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEnvelope {
    /// Message ID (unencrypted for tracking)
    pub id: Uuid,

    /// Sender ID (unencrypted for routing)
    pub sender: PersonId,

    /// Recipient ID (unencrypted for routing)
    pub recipient: PersonId,

    /// Encrypted message content
    pub encrypted_content: Vec<u8>,

    /// Signature from sender (proves authenticity)
    pub signature: Vec<u8>,

    /// When was this created? (coarse for privacy)
    pub created_at: CoarseTimestamp,

    /// Hop count (prevent infinite loops)
    pub hop_count: u8,

    /// Nonce for encryption (included in envelope)
    pub nonce: Vec<u8>,
}

impl Message {
    /// Create a new text message
    pub fn new_text(
        sender: PersonId,
        recipient: PersonId,
        content: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            sender,
            recipient,
            message_type: MessageType::Text {
                content: content.into(),
            },
            created_at: CoarseTimestamp::now(),
            expires_at: None,
            status: MessageStatus::Draft,
            hop_count: 0,
        }
    }

    /// Create an emergency message
    pub fn new_emergency(
        sender: PersonId,
        recipient: PersonId,
        request: EmergencyRequest,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            sender,
            recipient,
            message_type: MessageType::Emergency { request },
            created_at: CoarseTimestamp::now(),
            expires_at: Some(CoarseTimestamp::from_datetime(
                chrono::Utc::now() + chrono::Duration::hours(6)
            )),
            status: MessageStatus::Draft,
            hop_count: 0,
        }
    }

    /// Create an intelligence message
    pub fn new_intelligence(
        sender: PersonId,
        recipient: PersonId,
        report: IntelligenceReport,
    ) -> Self {
        let expires_at = report.expires_at;

        Self {
            id: Uuid::new_v4(),
            sender,
            recipient,
            message_type: MessageType::Intelligence { report },
            created_at: CoarseTimestamp::now(),
            expires_at: Some(expires_at),
            status: MessageStatus::Draft,
            hop_count: 0,
        }
    }

    /// Create a connection request
    pub fn new_connection_request(
        sender: PersonId,
        recipient: PersonId,
        fingerprint: crate::Fingerprint,
        public_key: Vec<u8>,
        introduction: Option<PersonId>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            sender,
            recipient,
            message_type: MessageType::ConnectionRequest {
                fingerprint,
                public_key,
                introduction,
            },
            created_at: CoarseTimestamp::now(),
            expires_at: Some(CoarseTimestamp::from_datetime(
                chrono::Utc::now() + chrono::Duration::days(7)
            )),
            status: MessageStatus::Draft,
            hop_count: 0,
        }
    }

    /// Check if message has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires) = self.expires_at {
            expires.is_expired(chrono::Duration::zero())
        } else {
            false
        }
    }

    /// Mark as sent
    pub fn mark_sent(&mut self) {
        self.status = MessageStatus::Sent;
    }

    /// Mark as delivered
    pub fn mark_delivered(&mut self) {
        if self.status == MessageStatus::Sent {
            self.status = MessageStatus::Delivered;
        }
    }

    /// Mark as read
    pub fn mark_read(&mut self) {
        if self.status == MessageStatus::Delivered {
            self.status = MessageStatus::Read;
        }
    }

    /// Increment hop count
    pub fn increment_hops(&mut self) {
        self.hop_count += 1;
    }

    /// Check if this is an emergency message
    pub fn is_emergency(&self) -> bool {
        matches!(
            self.message_type,
            MessageType::Emergency { .. } | MessageType::EmergencyResponse { .. }
        )
    }

    /// Get priority for message delivery (higher = more urgent)
    pub fn priority(&self) -> u32 {
        match &self.message_type {
            MessageType::Emergency { request } => {
                // Emergency messages get highest priority
                request.urgency as u32 * 1000
            }
            MessageType::EmergencyResponse { .. } => 3000,
            MessageType::Intelligence { report } => {
                report.urgency as u32 * 100
            }
            MessageType::ConnectionRequest { .. } => 500,
            MessageType::ConnectionAccepted { .. } => 400,
            _ => 100,
        }
    }
}

impl MessageEnvelope {
    /// Create an envelope from a message (requires encryption)
    pub fn new(
        message: &Message,
        encrypted_content: Vec<u8>,
        signature: Vec<u8>,
        nonce: Vec<u8>,
    ) -> Self {
        Self {
            id: message.id,
            sender: message.sender,
            recipient: message.recipient,
            encrypted_content,
            signature,
            created_at: message.created_at,
            hop_count: message.hop_count,
            nonce,
        }
    }

    /// Check if envelope signature is valid (requires sender's public key)
    pub fn verify_signature(&self, sender_public_key: &[u8]) -> crate::Result<bool> {
        use ed25519_dalek::{Signature, Verifier, VerifyingKey};

        // Reconstruct signed_data
        let mut signed_data = Vec::new();
        signed_data.extend_from_slice(self.id.as_bytes());
        signed_data.extend_from_slice(self.sender.0.as_bytes());
        signed_data.extend_from_slice(self.recipient.0.as_bytes());
        signed_data.extend_from_slice(&self.encrypted_content);

        // Verify signature
        let public_bytes: [u8; 32] = sender_public_key
            .try_into()
            .map_err(|_| crate::Error::Crypto("Invalid public key length".to_string()))?;

        let verifying_key = VerifyingKey::from_bytes(&public_bytes)
            .map_err(|e| crate::Error::Crypto(format!("Invalid public key: {}", e)))?;

        let signature_bytes: [u8; 64] = self.signature[..64]
            .try_into()
            .map_err(|_| crate::Error::Crypto("Invalid signature length".to_string()))?;

        let signature = Signature::from_bytes(&signature_bytes);

        Ok(verifying_key.verify(&signed_data, &signature).is_ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_message() {
        let sender = PersonId::new();
        let recipient = PersonId::new();

        let msg = Message::new_text(sender, recipient, "Hello, world!");

        assert_eq!(msg.sender, sender);
        assert_eq!(msg.recipient, recipient);
        assert!(matches!(msg.message_type, MessageType::Text { .. }));
        assert_eq!(msg.status, MessageStatus::Draft);
    }

    #[test]
    fn test_emergency_message() {
        let sender = PersonId::new();
        let recipient = PersonId::new();
        let emergency = crate::assistance::EmergencyRequest::new(
            Some(sender),
            vec![crate::assistance::EmergencyNeed::SafeShelter],
            crate::Region::new("Test"),
            crate::Urgency::Critical,
            2,
        );

        let msg = Message::new_emergency(sender, recipient, emergency);

        assert!(msg.is_emergency());
        assert!(msg.expires_at.is_some());
    }

    #[test]
    fn test_message_status_transitions() {
        let mut msg = Message::new_text(PersonId::new(), PersonId::new(), "Test");

        assert_eq!(msg.status, MessageStatus::Draft);

        msg.mark_sent();
        assert_eq!(msg.status, MessageStatus::Sent);

        msg.mark_delivered();
        assert_eq!(msg.status, MessageStatus::Delivered);

        msg.mark_read();
        assert_eq!(msg.status, MessageStatus::Read);
    }

    #[test]
    fn test_message_priority() {
        let sender = PersonId::new();
        let recipient = PersonId::new();

        let text = Message::new_text(sender, recipient, "Hello");
        let emergency = Message::new_emergency(
            sender,
            recipient,
            crate::assistance::EmergencyRequest::new(
                Some(sender),
                vec![crate::assistance::EmergencyNeed::ImmediateDanger],
                crate::Region::new("Test"),
                crate::Urgency::Critical,
                1,
            ),
        );

        assert!(emergency.priority() > text.priority());
    }

    #[test]
    fn test_hop_count() {
        let mut msg = Message::new_text(PersonId::new(), PersonId::new(), "Test");

        assert_eq!(msg.hop_count, 0);

        msg.increment_hops();
        assert_eq!(msg.hop_count, 1);

        msg.increment_hops();
        assert_eq!(msg.hop_count, 2);
    }

    #[test]
    fn test_connection_request() {
        let sender = PersonId::new();
        let recipient = PersonId::new();
        let fingerprint = crate::Fingerprint::new([0u8; 32]);
        let public_key = vec![1, 2, 3, 4];

        let msg = Message::new_connection_request(
            sender,
            recipient,
            fingerprint,
            public_key,
            None,
        );

        assert!(matches!(msg.message_type, MessageType::ConnectionRequest { .. }));
        assert!(msg.expires_at.is_some());
    }

    #[test]
    fn test_message_expiration() {
        let sender = PersonId::new();
        let recipient = PersonId::new();

        // Create message that expires in the past
        let mut msg = Message::new_text(sender, recipient, "Test");
        msg.expires_at = Some(CoarseTimestamp::from_datetime(
            chrono::Utc::now() - chrono::Duration::hours(1)
        ));

        assert!(msg.is_expired());

        // Create message that expires in the future
        let mut msg2 = Message::new_text(sender, recipient, "Test");
        msg2.expires_at = Some(CoarseTimestamp::from_datetime(
            chrono::Utc::now() + chrono::Duration::hours(1)
        ));

        assert!(!msg2.is_expired());
    }
}
