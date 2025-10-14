//! QR code generation for contact sharing
//!
//! QR codes provide a secure, convenient way to share contact information
//! in person. Scan a QR code to add someone as a contact.

use crate::{Fingerprint, PersonId, Result};
use serde::{Deserialize, Serialize};

/// Contact card - information shared via QR code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactCard {
    /// Display name
    pub name: String,

    /// Person ID
    pub id: PersonId,

    /// Public signing key
    pub public_key: Vec<u8>,

    /// Fingerprint (for verification)
    pub fingerprint: Fingerprint,

    /// Veilid route (if available)
    pub veilid_route: Option<Vec<u8>>,

    /// Expiration timestamp (optional, for temporary sharing)
    pub expires_at: Option<i64>,
}

impl ContactCard {
    /// Create a new contact card
    pub fn new(
        name: impl Into<String>,
        id: PersonId,
        public_key: Vec<u8>,
        fingerprint: Fingerprint,
    ) -> Self {
        Self {
            name: name.into(),
            id,
            public_key,
            fingerprint,
            veilid_route: None,
            expires_at: None,
        }
    }

    /// Add Veilid route information
    pub fn with_veilid_route(mut self, route: Vec<u8>) -> Self {
        self.veilid_route = Some(route);
        self
    }

    /// Set expiration time (Unix timestamp)
    pub fn with_expiration(mut self, expires_at: i64) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    /// Encode as base64 (for QR code)
    pub fn encode(&self) -> Result<String> {
        let serialized = bincode::serialize(self)
            .map_err(|e| crate::Error::Serialization(e.to_string()))?;

        Ok(base64::encode(&serialized))
    }

    /// Decode from base64
    pub fn decode(encoded: &str) -> Result<Self> {
        let bytes = base64::decode(encoded)
            .map_err(|e| crate::Error::Serialization(format!("Base64 decode failed: {}", e)))?;

        let card: ContactCard = bincode::deserialize(&bytes)
            .map_err(|e| crate::Error::Serialization(e.to_string()))?;

        // Check expiration
        if let Some(expires) = card.expires_at {
            let now = chrono::Utc::now().timestamp();
            if now > expires {
                return Err(crate::Error::Invalid("Contact card has expired".to_string()));
            }
        }

        Ok(card)
    }

    /// Check if this card has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires) = self.expires_at {
            let now = chrono::Utc::now().timestamp();
            now > expires
        } else {
            false
        }
    }

    /// Get verification words
    pub fn verification_words(&self) -> [&'static str; 3] {
        self.fingerprint.to_words()
    }
}

/// QR code format for contact sharing
pub struct ContactQR {
    card: ContactCard,
}

impl ContactQR {
    /// Create a new QR code from a contact card
    pub fn new(card: ContactCard) -> Self {
        Self { card }
    }

    /// Get the data to encode in the QR code
    pub fn data(&self) -> Result<String> {
        // Format: railroad://contact/BASE64_DATA
        let encoded = self.card.encode()?;
        Ok(format!("railroad://contact/{}", encoded))
    }

    /// Parse QR code data
    pub fn parse(data: &str) -> Result<ContactCard> {
        // Check prefix
        if !data.starts_with("railroad://contact/") {
            return Err(crate::Error::Invalid(
                "Invalid QR code format (wrong prefix)".to_string(),
            ));
        }

        // Extract base64 data
        let encoded = data
            .strip_prefix("railroad://contact/")
            .ok_or_else(|| crate::Error::Invalid("Invalid QR code format".to_string()))?;

        ContactCard::decode(encoded)
    }

    /// Get the contact card
    pub fn card(&self) -> &ContactCard {
        &self.card
    }
}

/// Helper to generate QR code as ASCII art (for terminal display)
pub fn generate_ascii_qr(data: &str) -> String {
    // TODO: Use qrcode crate to generate actual QR codes
    // For now, return a placeholder
    format!(
        r#"
┌─────────────────┐
│  QR CODE        │
│                 │
│  ███ ██ ███     │
│  █ ███ █ █      │
│  ███ ██ ███     │
│                 │
│  Data: {}...    │
└─────────────────┘
    "#,
        &data[..data.len().min(20)]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_card() -> ContactCard {
        ContactCard::new(
            "Alice",
            PersonId::new(),
            vec![1, 2, 3, 4],
            Fingerprint::new([0u8; 32]),
        )
    }

    #[test]
    fn test_contact_card_encode_decode() {
        let card = test_card();

        let encoded = card.encode().unwrap();
        assert!(!encoded.is_empty());

        let decoded = ContactCard::decode(&encoded).unwrap();
        assert_eq!(decoded.name, card.name);
        assert_eq!(decoded.id, card.id);
        assert_eq!(decoded.public_key, card.public_key);
    }

    #[test]
    fn test_contact_card_expiration() {
        let past = chrono::Utc::now().timestamp() - 3600; // 1 hour ago
        let future = chrono::Utc::now().timestamp() + 3600; // 1 hour from now

        let expired = test_card().with_expiration(past);
        assert!(expired.is_expired());

        let valid = test_card().with_expiration(future);
        assert!(!valid.is_expired());
    }

    #[test]
    fn test_contact_card_expiration_check_on_decode() {
        let past = chrono::Utc::now().timestamp() - 3600;
        let expired_card = test_card().with_expiration(past);

        let encoded = expired_card.encode().unwrap();

        // Should fail to decode expired card
        let result = ContactCard::decode(&encoded);
        assert!(result.is_err());
    }

    #[test]
    fn test_contact_qr_format() {
        let card = test_card();
        let qr = ContactQR::new(card);

        let data = qr.data().unwrap();
        assert!(data.starts_with("railroad://contact/"));
    }

    #[test]
    fn test_contact_qr_parse() {
        let card = test_card();
        let qr = ContactQR::new(card.clone());

        let data = qr.data().unwrap();

        let parsed_card = ContactQR::parse(&data).unwrap();
        assert_eq!(parsed_card.name, card.name);
        assert_eq!(parsed_card.id, card.id);
    }

    #[test]
    fn test_contact_qr_invalid_prefix() {
        let result = ContactQR::parse("invalid://data");
        assert!(result.is_err());
    }

    #[test]
    fn test_verification_words() {
        let card = test_card();
        let words = card.verification_words();

        assert_eq!(words.len(), 3);
    }

    #[test]
    fn test_ascii_qr_generation() {
        let ascii = generate_ascii_qr("test data");
        assert!(ascii.contains("QR CODE"));
        assert!(ascii.contains("test"));
    }
}
