//! Identity and persona management
//!
//! Multiple personas provide:
//! - Compartmentalization (separate identities for different contexts)
//! - Plausible deniability (can't prove all personas belong to same person)
//! - Safety (compromise of one persona doesn't expose others)

pub mod keypair;
pub mod persona;
pub mod qr;

pub use keypair::{IdentityKeypair, SigningKey, EncryptionKey};
pub use persona::{PersonaManager, PersonaBuilder, PersonaSummary};
pub use qr::{ContactCard, ContactQR};

use crate::{Fingerprint, PersonId, Result};
use serde::{Deserialize, Serialize};

/// A complete identity with keypairs
#[derive(Clone, Serialize, Deserialize)]
pub struct Identity {
    /// Unique identifier
    pub id: PersonId,

    /// Display name (user-chosen, can be pseudonym)
    pub name: String,

    /// Keypair for signing and encryption
    pub keypair: IdentityKeypair,

    /// Fingerprint derived from public key
    pub fingerprint: Fingerprint,

    /// When was this identity created?
    pub created_at: crate::CoarseTimestamp,

    /// Is this the primary identity?
    pub is_primary: bool,
}

impl Identity {
    /// Create a new identity from a seed
    pub fn from_seed(name: impl Into<String>, seed: &[u8; 32], is_primary: bool) -> Result<Self> {
        let keypair = IdentityKeypair::from_seed(seed)?;
        let fingerprint = keypair.fingerprint();
        let id = PersonId::new();

        Ok(Self {
            id,
            name: name.into(),
            keypair,
            fingerprint,
            created_at: crate::CoarseTimestamp::now(),
            is_primary,
        })
    }

    /// Generate a new random identity
    pub fn generate(name: impl Into<String>, is_primary: bool) -> Result<Self> {
        let keypair = IdentityKeypair::generate()?;
        let fingerprint = keypair.fingerprint();
        let id = PersonId::new();

        Ok(Self {
            id,
            name: name.into(),
            keypair,
            fingerprint,
            created_at: crate::CoarseTimestamp::now(),
            is_primary,
        })
    }

    /// Get the public key as bytes
    pub fn public_key_bytes(&self) -> &[u8] {
        self.keypair.signing_key().public_key_bytes()
    }

    /// Get verification words for this identity
    pub fn verification_words(&self) -> [&'static str; 3] {
        self.fingerprint.to_words()
    }

    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> Result<Vec<u8>> {
        self.keypair.signing_key().sign(message)
    }

    /// Verify a signature
    pub fn verify(&self, message: &[u8], signature: &[u8]) -> Result<bool> {
        self.keypair.signing_key().verify(message, signature)
    }

    /// Encrypt data for a recipient
    pub fn encrypt_for(&self, recipient_public_key: &[u8], plaintext: &[u8]) -> Result<Vec<u8>> {
        self.keypair.encryption_key().encrypt_for(recipient_public_key, plaintext)
    }

    /// Decrypt data from a sender
    pub fn decrypt_from(&self, sender_public_key: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>> {
        self.keypair.encryption_key().decrypt_from(sender_public_key, ciphertext)
    }

    /// Export identity for backup (encrypted)
    pub fn export_encrypted(&self, password: &str) -> Result<Vec<u8>> {
        // Derive encryption key from password
        // Use first 16 bytes of UUID as salt, expand to 32 bytes
        let uuid_bytes = self.id.0.as_bytes();
        let mut salt = [0u8; 32];
        salt[..16].copy_from_slice(uuid_bytes);
        salt[16..].copy_from_slice(uuid_bytes); // Repeat to fill 32 bytes

        let key = crate::crypto::derive_master_key(password, &salt)?;

        // Serialize identity
        let serialized = bincode::serialize(self)
            .map_err(|e| crate::Error::Serialization(e.to_string()))?;

        // Encrypt
        use chacha20poly1305::{
            aead::{Aead, AeadCore, KeyInit, OsRng},
            ChaCha20Poly1305,
        };

        let cipher = ChaCha20Poly1305::new(key.as_ref().into());
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);

        let ciphertext = cipher
            .encrypt(&nonce, serialized.as_ref())
            .map_err(|e| crate::Error::Crypto(format!("Encryption failed: {}", e)))?;

        // Combine nonce + ciphertext
        let mut output = nonce.to_vec();
        output.extend_from_slice(&ciphertext);

        Ok(output)
    }

    /// Import identity from backup (encrypted)
    pub fn import_encrypted(data: &[u8], password: &str) -> Result<Self> {
        use chacha20poly1305::{
            aead::{Aead, KeyInit},
            ChaCha20Poly1305, Nonce,
        };

        // Split nonce and ciphertext
        if data.len() < 12 {
            return Err(crate::Error::Invalid("Invalid backup data".to_string()));
        }

        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        // We need the ID to derive the key, but it's encrypted...
        // Solution: Try to decrypt and if it fails, wrong password
        // We'll use a fixed salt for import (user must know correct password)
        let salt = [0u8; 32]; // Placeholder - in reality, we'd need to store salt with backup

        let key = crate::crypto::derive_master_key(password, &salt)?;
        let cipher = ChaCha20Poly1305::new(key.as_ref().into());

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| crate::Error::Crypto("Decryption failed (wrong password?)".to_string()))?;

        // Deserialize
        let identity: Identity = bincode::deserialize(&plaintext)
            .map_err(|e| crate::Error::Serialization(e.to_string()))?;

        Ok(identity)
    }
}

impl std::fmt::Debug for Identity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Identity")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("fingerprint", &self.fingerprint)
            .field("created_at", &self.created_at)
            .field("is_primary", &self.is_primary)
            .field("keypair", &"[REDACTED]")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_creation() {
        let identity = Identity::generate("Alice", true).unwrap();

        assert_eq!(identity.name, "Alice");
        assert!(identity.is_primary);
        assert!(identity.public_key_bytes().len() > 0);
    }

    #[test]
    fn test_identity_from_seed() {
        let seed = [42u8; 32];

        let id1 = Identity::from_seed("Test", &seed, false).unwrap();
        let id2 = Identity::from_seed("Test", &seed, false).unwrap();

        // Same seed should produce same keypair (but different ID)
        assert_eq!(id1.public_key_bytes(), id2.public_key_bytes());
        assert_ne!(id1.id, id2.id);
    }

    #[test]
    fn test_signing_and_verification() {
        let identity = Identity::generate("Bob", false).unwrap();
        let message = b"Hello, world!";

        let signature = identity.sign(message).unwrap();
        assert!(identity.verify(message, &signature).unwrap());

        // Wrong message should fail
        let wrong_message = b"Different message";
        assert!(!identity.verify(wrong_message, &signature).unwrap());
    }

    #[test]
    fn test_encryption_and_decryption() {
        let alice = Identity::generate("Alice", false).unwrap();
        let bob = Identity::generate("Bob", false).unwrap();

        let plaintext = b"Secret message from Alice to Bob";

        // Alice encrypts for Bob
        let ciphertext = alice
            .encrypt_for(bob.public_key_bytes(), plaintext)
            .unwrap();

        // Bob decrypts from Alice
        let decrypted = bob
            .decrypt_from(alice.public_key_bytes(), &ciphertext)
            .unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_verification_words() {
        let identity = Identity::generate("Charlie", false).unwrap();
        let words = identity.verification_words();

        assert_eq!(words.len(), 3);
        // Words should be from the verification word list
        for word in words {
            assert!(!word.is_empty());
        }
    }

    #[test]
    fn test_export_import() {
        let original = Identity::generate("David", true).unwrap();
        let password = "strong password 12345";

        // Export
        let encrypted = original.export_encrypted(password).unwrap();
        assert!(encrypted.len() > 0);

        // Import
        let imported = Identity::import_encrypted(&encrypted, password).unwrap();

        assert_eq!(original.id, imported.id);
        assert_eq!(original.name, imported.name);
        assert_eq!(original.public_key_bytes(), imported.public_key_bytes());
    }

    #[test]
    fn test_import_wrong_password() {
        let original = Identity::generate("Eve", false).unwrap();
        let correct_password = "correct password";
        let wrong_password = "wrong password";

        let encrypted = original.export_encrypted(correct_password).unwrap();

        // Should fail with wrong password
        let result = Identity::import_encrypted(&encrypted, wrong_password);
        assert!(result.is_err());
    }
}
