//! Messaging protocol for secure communication
//!
//! This module handles:
//! - Message types (text, assistance requests, intelligence)
//! - End-to-end encryption (Double Ratchet)
//! - Message routing through Veilid network
//! - Offline message queue
//! - Read receipts and delivery status

pub mod protocol;
pub mod encryption;
pub mod routing;

pub use protocol::{Message, MessageType, MessageStatus, MessageEnvelope};
pub use encryption::{EncryptedMessage, MessageKey};
pub use routing::{MessageRouter, Route};
