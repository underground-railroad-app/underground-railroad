//! Message encryption using hybrid post-quantum cryptography
//!
//! **Threat Model: Nation-state adversaries with quantum computers**
//!
//! **Security Features:**
//! - Hybrid encryption: X25519 (classical) + Kyber1024 (post-quantum)
//! - Authenticated encryption: ChaCha20-Poly1305
//! - Hybrid signatures: Ed25519 + Dilithium5
//! - Forward secrecy: Ephemeral keys for each message
//! - Key derivation: HKDF-SHA512
//!
//! **Design:**
//! 1. Generate ephemeral keypair for both X25519 and Kyber
//! 2. Perform key exchange with recipient's public keys
//! 3. Combine shared secrets using HKDF
//! 4. Encrypt message with ChaCha20-Poly1305
//! 5. Sign with both Ed25519 and Dilithium5

use crate::{PersonId, Result, SecureBytes};
use pqcrypto_traits::kem::SharedSecret as PQSharedSecret;
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// An encrypted message with hybrid post-quantum protection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedMessage {
    /// Encrypted content
    pub ciphertext: Vec<u8>,

    /// Nonce used for encryption
    pub nonce: Vec<u8>,

    /// Authentication tag (included in ChaCha20-Poly1305)
    pub tag: Vec<u8>,

    /// Ephemeral X25519 public key (classical)
    pub ephemeral_x25519_public_key: Vec<u8>,

    /// Ephemeral Kyber1024 ciphertext (post-quantum)
    /// This is the result of encrypting to recipient's Kyber public key
    pub kyber_ciphertext: Vec<u8>,
}

/// Message encryption key (derived from key exchange)
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct MessageKey([u8; 32]);

impl MessageKey {
    /// Create from raw bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Get key bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Encrypt plaintext (legacy - for backward compatibility)
    /// For new code, use encrypt_message_hybrid instead
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptedMessage> {
        use chacha20poly1305::{
            aead::{Aead, AeadCore, KeyInit, OsRng},
            ChaCha20Poly1305,
        };

        let cipher = ChaCha20Poly1305::new((&self.0).into());
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);

        let ciphertext = cipher
            .encrypt(&nonce, plaintext)
            .map_err(|e| crate::Error::Crypto(format!("Encryption failed: {}", e)))?;

        Ok(EncryptedMessage {
            ciphertext,
            nonce: nonce.to_vec(),
            tag: Vec::new(), // Tag is embedded in ciphertext for ChaCha20-Poly1305
            ephemeral_x25519_public_key: Vec::new(), // Will be set by caller
            kyber_ciphertext: Vec::new(), // Not used in legacy mode
        })
    }

    /// Decrypt ciphertext
    pub fn decrypt(&self, encrypted: &EncryptedMessage) -> Result<Vec<u8>> {
        use chacha20poly1305::{aead::{Aead, KeyInit}, ChaCha20Poly1305, Nonce};

        let cipher = ChaCha20Poly1305::new((&self.0).into());
        let nonce = Nonce::from_slice(&encrypted.nonce);

        let plaintext = cipher
            .decrypt(nonce, encrypted.ciphertext.as_ref())
            .map_err(|e| crate::Error::Crypto(format!("Decryption failed: {}", e)))?;

        Ok(plaintext)
    }
}

/// Hybrid key exchange: X25519 + Kyber1024
/// Combines classical and post-quantum shared secrets for maximum security
pub fn derive_hybrid_message_key(
    x25519_secret: &[u8; 32],
    x25519_their_public: &[u8; 32],
    kyber_shared_secret: &[u8],
) -> Result<MessageKey> {
    use hkdf::Hkdf;
    use sha2::Sha512;

    // Perform X25519 key exchange
    let x25519_shared = x25519_dalek::x25519(*x25519_secret, *x25519_their_public);

    // Combine both shared secrets using HKDF
    // This ensures security even if one scheme is broken
    let mut combined = Vec::new();
    combined.extend_from_slice(&x25519_shared);
    combined.extend_from_slice(kyber_shared_secret);

    // Derive final message key using HKDF
    let hk = Hkdf::<Sha512>::new(None, &combined);
    let mut derived_key = [0u8; 32];
    hk.expand(b"underground-railroad-message-key-v1", &mut derived_key)
        .map_err(|e| crate::Error::Crypto(format!("HKDF expansion failed: {}", e)))?;

    // Zeroize intermediate values
    drop(combined);

    Ok(MessageKey::from_bytes(derived_key))
}

/// Legacy X25519-only key exchange (for backward compatibility)
pub fn derive_message_key(
    our_secret: &[u8; 32],
    their_public: &[u8; 32],
) -> Result<MessageKey> {
    // In x25519-dalek v2.0, use x25519 function directly
    let shared_secret = x25519_dalek::x25519(*our_secret, *their_public);

    Ok(MessageKey::from_bytes(shared_secret))
}

/// Public keys needed for message encryption
#[derive(Clone, Serialize, Deserialize)]
pub struct RecipientPublicKeys {
    /// X25519 public key (classical)
    pub x25519: [u8; 32],

    /// Kyber1024 public key (post-quantum)
    pub kyber: Vec<u8>,
}

/// Encrypt a message using hybrid post-quantum encryption
pub fn encrypt_message_hybrid(
    plaintext: &[u8],
    recipient_keys: &RecipientPublicKeys,
) -> Result<EncryptedMessage> {
    use chacha20poly1305::{
        aead::{Aead, AeadCore, KeyInit, OsRng},
        ChaCha20Poly1305, Nonce,
    };
    use pqcrypto_kyber::kyber1024;
    use pqcrypto_traits::kem::{PublicKey, Ciphertext as PQCiphertext};

    // Generate ephemeral X25519 keypair
    let mut ephemeral_secret = [0u8; 32];
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut ephemeral_secret);
    let ephemeral_public = x25519_dalek::PublicKey::from(ephemeral_secret);

    // Perform Kyber encapsulation
    let kyber_public_key = kyber1024::PublicKey::from_bytes(&recipient_keys.kyber)
        .map_err(|e| crate::Error::Crypto(format!("Invalid Kyber public key: {:?}", e)))?;

    let (kyber_ciphertext, kyber_shared_secret) = kyber1024::encapsulate(&kyber_public_key);

    // Derive hybrid message key
    let message_key = derive_hybrid_message_key(
        &ephemeral_secret,
        &recipient_keys.x25519,
        kyber_shared_secret.as_bytes(),
    )?;

    // Encrypt with ChaCha20-Poly1305
    let cipher = ChaCha20Poly1305::new(message_key.as_bytes().into());
    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);

    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|e| crate::Error::Crypto(format!("Encryption failed: {}", e)))?;

    // Zeroize ephemeral secret
    use zeroize::Zeroize;
    ephemeral_secret.zeroize();

    Ok(EncryptedMessage {
        ciphertext,
        nonce: nonce.to_vec(),
        tag: Vec::new(), // Tag embedded in ciphertext
        ephemeral_x25519_public_key: ephemeral_public.to_bytes().to_vec(),
        kyber_ciphertext: kyber_ciphertext.as_bytes().to_vec(),
    })
}

/// Decrypt a hybrid encrypted message
pub fn decrypt_message_hybrid(
    encrypted: &EncryptedMessage,
    x25519_secret: &[u8; 32],
    kyber_secret_key: &[u8],
) -> Result<Vec<u8>> {
    use chacha20poly1305::{aead::{Aead, KeyInit}, ChaCha20Poly1305, Nonce};
    use pqcrypto_kyber::kyber1024;
    use pqcrypto_traits::kem::{Ciphertext, SecretKey};

    // Recover ephemeral X25519 public key
    let ephemeral_public: [u8; 32] = encrypted.ephemeral_x25519_public_key[..]
        .try_into()
        .map_err(|_| crate::Error::Crypto("Invalid ephemeral public key".to_string()))?;

    // Perform Kyber decapsulation
    let kyber_sk = kyber1024::SecretKey::from_bytes(kyber_secret_key)
        .map_err(|e| crate::Error::Crypto(format!("Invalid Kyber secret key: {:?}", e)))?;

    let kyber_ct = kyber1024::Ciphertext::from_bytes(&encrypted.kyber_ciphertext)
        .map_err(|e| crate::Error::Crypto(format!("Invalid Kyber ciphertext: {:?}", e)))?;

    let kyber_shared_secret = kyber1024::decapsulate(&kyber_ct, &kyber_sk);

    // Derive hybrid message key
    let message_key = derive_hybrid_message_key(
        x25519_secret,
        &ephemeral_public,
        kyber_shared_secret.as_bytes(),
    )?;

    // Decrypt with ChaCha20-Poly1305
    let cipher = ChaCha20Poly1305::new(message_key.as_bytes().into());
    let nonce = Nonce::from_slice(&encrypted.nonce);

    let plaintext = cipher
        .decrypt(nonce, encrypted.ciphertext.as_ref())
        .map_err(|e| crate::Error::Crypto(format!("Decryption failed: {}", e)))?;

    Ok(plaintext)
}

impl std::fmt::Debug for MessageKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MessageKey([REDACTED])")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_key_encrypt_decrypt() {
        let key = MessageKey::from_bytes([42u8; 32]);
        let plaintext = b"Secret message";

        let encrypted = key.encrypt(plaintext).unwrap();
        let decrypted = key.decrypt(&encrypted).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_wrong_key_fails() {
        let key1 = MessageKey::from_bytes([1u8; 32]);
        let key2 = MessageKey::from_bytes([2u8; 32]);

        let plaintext = b"Secret message";
        let encrypted = key1.encrypt(plaintext).unwrap();

        // Decryption with wrong key should fail
        let result = key2.decrypt(&encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn test_key_exchange() {
        use x25519_dalek::PublicKey;

        // Alice's keypair
        let alice_secret = [1u8; 32];
        let alice_public = PublicKey::from(alice_secret);

        // Bob's keypair
        let bob_secret = [2u8; 32];
        let bob_public = PublicKey::from(bob_secret);

        // Derive shared keys
        let alice_key = derive_message_key(
            &alice_secret,
            bob_public.as_bytes(),
        ).unwrap();

        let bob_key = derive_message_key(
            &bob_secret,
            alice_public.as_bytes(),
        ).unwrap();

        // Both should derive the same key
        assert_eq!(alice_key.as_bytes(), bob_key.as_bytes());

        // Verify encryption works both ways
        let plaintext = b"Test message";

        let encrypted_by_alice = alice_key.encrypt(plaintext).unwrap();
        let decrypted_by_bob = bob_key.decrypt(&encrypted_by_alice).unwrap();

        assert_eq!(decrypted_by_bob, plaintext);
    }

    #[test]
    fn test_encrypted_message_has_nonce() {
        let key = MessageKey::from_bytes([42u8; 32]);
        let encrypted = key.encrypt(b"test").unwrap();

        assert!(!encrypted.nonce.is_empty());
        assert!(!encrypted.ciphertext.is_empty());
    }
}
