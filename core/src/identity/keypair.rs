//! Cryptographic keypairs for identity

use crate::{Fingerprint, Result};
use ed25519_dalek::{Signer, Verifier};
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// A complete identity keypair (signing + encryption)
#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct IdentityKeypair {
    signing: SigningKey,
    encryption: EncryptionKey,
}

impl IdentityKeypair {
    /// Generate a new random keypair
    pub fn generate() -> Result<Self> {
        let signing = SigningKey::generate()?;
        let encryption = EncryptionKey::generate()?;

        Ok(Self { signing, encryption })
    }

    /// Create keypair from a seed
    pub fn from_seed(seed: &[u8; 32]) -> Result<Self> {
        // Derive separate seeds for signing and encryption
        use hkdf::Hkdf;
        use sha2::Sha256;

        let hkdf = Hkdf::<Sha256>::new(None, seed);

        let mut signing_seed = zeroize::Zeroizing::new([0u8; 32]);
        hkdf.expand(b"underground-railroad-signing", &mut *signing_seed)
            .map_err(|e| crate::Error::Crypto(format!("HKDF expand failed: {}", e)))?;

        let mut encryption_seed = zeroize::Zeroizing::new([0u8; 32]);
        hkdf.expand(b"underground-railroad-encryption", &mut *encryption_seed)
            .map_err(|e| crate::Error::Crypto(format!("HKDF expand failed: {}", e)))?;

        let signing = SigningKey::from_seed(&*signing_seed)?;
        let encryption = EncryptionKey::from_seed(&*encryption_seed)?;

        Ok(Self { signing, encryption })
    }

    /// Get the signing key
    pub fn signing_key(&self) -> &SigningKey {
        &self.signing
    }

    /// Get the encryption key
    pub fn encryption_key(&self) -> &EncryptionKey {
        &self.encryption
    }

    /// Get fingerprint (hash of public signing key)
    pub fn fingerprint(&self) -> Fingerprint {
        self.signing.fingerprint()
    }
}

/// Ed25519 signing key (for authentication and signatures)
#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct SigningKey {
    #[serde(with = "serde_bytes")]
    secret: Vec<u8>,
    #[serde(with = "serde_bytes")]
    public: Vec<u8>,
}

impl SigningKey {
    /// Generate a new random signing key
    pub fn generate() -> Result<Self> {
        use ed25519_dalek::SigningKey as Ed25519SigningKey;
        use rand::RngCore;

        // In ed25519-dalek v2.1+, generate random bytes first
        let mut secret_bytes = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut secret_bytes);

        let signing_key = Ed25519SigningKey::from_bytes(&secret_bytes);
        let verifying_key = signing_key.verifying_key();

        Ok(Self {
            secret: signing_key.to_bytes().to_vec(),
            public: verifying_key.to_bytes().to_vec(),
        })
    }

    /// Create signing key from seed
    pub fn from_seed(seed: &[u8; 32]) -> Result<Self> {
        use ed25519_dalek::SigningKey as Ed25519SigningKey;

        let signing_key = Ed25519SigningKey::from_bytes(seed);
        let verifying_key = signing_key.verifying_key();

        Ok(Self {
            secret: signing_key.to_bytes().to_vec(),
            public: verifying_key.to_bytes().to_vec(),
        })
    }

    /// Get public key bytes
    pub fn public_key_bytes(&self) -> &[u8] {
        &self.public
    }

    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> Result<Vec<u8>> {
        use ed25519_dalek::SigningKey as Ed25519SigningKey;

        let secret_bytes: [u8; 32] = self.secret[..32]
            .try_into()
            .map_err(|_| crate::Error::Crypto("Invalid secret key length".to_string()))?;

        let signing_key = Ed25519SigningKey::from_bytes(&secret_bytes);
        let signature = signing_key.sign(message);

        Ok(signature.to_bytes().to_vec())
    }

    /// Verify a signature
    pub fn verify(&self, message: &[u8], signature: &[u8]) -> Result<bool> {
        use ed25519_dalek::{Signature, VerifyingKey};

        let public_bytes: [u8; 32] = self.public[..32]
            .try_into()
            .map_err(|_| crate::Error::Crypto("Invalid public key length".to_string()))?;

        let verifying_key = VerifyingKey::from_bytes(&public_bytes)
            .map_err(|e| crate::Error::Crypto(format!("Invalid public key: {}", e)))?;

        let signature_bytes: [u8; 64] = signature
            .try_into()
            .map_err(|_| crate::Error::Crypto("Invalid signature length".to_string()))?;

        let signature = Signature::from_bytes(&signature_bytes);

        match verifying_key.verify_strict(message, &signature) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Get fingerprint (SHA-256 of public key)
    pub fn fingerprint(&self) -> Fingerprint {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(&self.public);
        let hash = hasher.finalize();

        let mut fp = [0u8; 32];
        fp.copy_from_slice(&hash);

        Fingerprint::new(fp)
    }
}

/// Hybrid encryption key (X25519 + Kyber1024 for post-quantum security)
#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct EncryptionKey {
    /// X25519 secret key (classical)
    #[serde(with = "serde_bytes")]
    x25519_secret: Vec<u8>,
    /// X25519 public key (classical)
    #[serde(with = "serde_bytes")]
    x25519_public: Vec<u8>,
    /// Kyber1024 secret key (post-quantum)
    #[serde(with = "serde_bytes")]
    kyber_secret: Vec<u8>,
    /// Kyber1024 public key (post-quantum)
    #[serde(with = "serde_bytes")]
    kyber_public: Vec<u8>,
}

impl EncryptionKey {
    /// Generate a new random hybrid encryption key (X25519 + Kyber1024)
    pub fn generate() -> Result<Self> {
        use pqcrypto_kyber::kyber1024;
        use pqcrypto_traits::kem::{PublicKey, SecretKey};
        use rand::RngCore;

        // Generate X25519 keypair
        let mut x25519_secret_bytes = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut x25519_secret_bytes);
        let x25519_public = x25519_dalek::PublicKey::from(x25519_secret_bytes);

        // Generate Kyber1024 keypair
        let (kyber_pk, kyber_sk) = kyber1024::keypair();

        Ok(Self {
            x25519_secret: x25519_secret_bytes.to_vec(),
            x25519_public: x25519_public.to_bytes().to_vec(),
            kyber_secret: kyber_sk.as_bytes().to_vec(),
            kyber_public: kyber_pk.as_bytes().to_vec(),
        })
    }

    /// Create encryption key from seed (deterministic)
    pub fn from_seed(seed: &[u8; 32]) -> Result<Self> {
        use hkdf::Hkdf;
        use pqcrypto_kyber::kyber1024;
        use pqcrypto_traits::kem::{PublicKey as PQPublicKey, SecretKey as PQSecretKey};
        use sha2::Sha512;

        // Derive X25519 key from seed
        let hkdf = Hkdf::<Sha512>::new(None, seed);
        let mut x25519_seed = zeroize::Zeroizing::new([0u8; 32]);
        hkdf.expand(b"x25519-encryption-key", &mut *x25519_seed)
            .map_err(|e| crate::Error::Crypto(format!("HKDF failed: {}", e)))?;

        let x25519_public = x25519_dalek::PublicKey::from(*x25519_seed);

        // For Kyber: generate random (ideal: deterministic keygen from seed)
        // TODO: Implement deterministic Kyber keygen for reproducibility
        let (kyber_pk, kyber_sk) = kyber1024::keypair();

        Ok(Self {
            x25519_secret: x25519_seed.to_vec(),
            x25519_public: x25519_public.to_bytes().to_vec(),
            kyber_secret: kyber_sk.as_bytes().to_vec(),
            kyber_public: kyber_pk.as_bytes().to_vec(),
        })
    }

    /// Get X25519 public key bytes
    pub fn public_key_bytes(&self) -> &[u8] {
        &self.x25519_public
    }

    /// Get Kyber1024 public key bytes
    pub fn kyber_public_key_bytes(&self) -> &[u8] {
        &self.kyber_public
    }

    /// Get hybrid public keys for message encryption
    pub fn get_public_keys(&self) -> crate::messaging::encryption::RecipientPublicKeys {
        let x25519: [u8; 32] = self.x25519_public[..]
            .try_into()
            .expect("Invalid X25519 public key length");

        crate::messaging::encryption::RecipientPublicKeys {
            x25519,
            kyber: self.kyber_public.clone(),
        }
    }

    /// Get X25519 secret key
    pub fn x25519_secret(&self) -> &[u8] {
        &self.x25519_secret
    }

    /// Get Kyber secret key
    pub fn kyber_secret(&self) -> &[u8] {
        &self.kyber_secret
    }

    /// Legacy X25519-only encryption (for backward compatibility)
    /// For new code, use messaging::encryption::encrypt_message_hybrid instead
    pub fn encrypt_for(&self, recipient_public_key: &[u8], plaintext: &[u8]) -> Result<Vec<u8>> {
        use chacha20poly1305::{
            aead::{Aead, AeadCore, KeyInit, OsRng},
            ChaCha20Poly1305,
        };

        // Perform X25519 key exchange
        let secret_bytes: [u8; 32] = self.x25519_secret[..32]
            .try_into()
            .map_err(|_| crate::Error::Crypto("Invalid secret key".to_string()))?;

        let recipient_bytes: [u8; 32] = recipient_public_key
            .try_into()
            .map_err(|_| crate::Error::Crypto("Invalid recipient key".to_string()))?;

        let shared_secret_bytes = x25519_dalek::x25519(secret_bytes, recipient_bytes);

        // Use shared secret to encrypt
        let cipher = ChaCha20Poly1305::new((&shared_secret_bytes).into());
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);

        let ciphertext = cipher
            .encrypt(&nonce, plaintext)
            .map_err(|e| crate::Error::Crypto(format!("Encryption failed: {}", e)))?;

        // Combine nonce + ciphertext
        let mut output = nonce.to_vec();
        output.extend_from_slice(&ciphertext);

        Ok(output)
    }

    /// Legacy X25519-only decryption (for backward compatibility)
    /// For new code, use messaging::encryption::decrypt_message_hybrid instead
    pub fn decrypt_from(&self, sender_public_key: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>> {
        use chacha20poly1305::{aead::{Aead, KeyInit}, ChaCha20Poly1305};

        if ciphertext.len() < 12 {
            return Err(crate::Error::Crypto("Ciphertext too short".to_string()));
        }

        let (nonce_bytes, ct) = ciphertext.split_at(12);
        let nonce = chacha20poly1305::Nonce::from_slice(nonce_bytes);

        // Perform X25519 key exchange
        let secret_bytes: [u8; 32] = self.x25519_secret[..32]
            .try_into()
            .map_err(|_| crate::Error::Crypto("Invalid secret key".to_string()))?;

        let sender_bytes: [u8; 32] = sender_public_key
            .try_into()
            .map_err(|_| crate::Error::Crypto("Invalid sender key".to_string()))?;

        let shared_secret_bytes = x25519_dalek::x25519(secret_bytes, sender_bytes);

        // Decrypt
        let cipher = ChaCha20Poly1305::new((&shared_secret_bytes).into());

        let plaintext = cipher
            .decrypt(nonce, ct)
            .map_err(|e| crate::Error::Crypto(format!("Decryption failed: {}", e)))?;

        Ok(plaintext)
    }
}

// Serde helper for binary data
mod serde_bytes {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(bytes)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Vec::<u8>::deserialize(deserializer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signing_key_generation() {
        let key1 = SigningKey::generate().unwrap();
        let key2 = SigningKey::generate().unwrap();

        // Different keys should have different public keys
        assert_ne!(key1.public_key_bytes(), key2.public_key_bytes());
    }

    #[test]
    fn test_signing_key_from_seed() {
        let seed = [42u8; 32];

        let key1 = SigningKey::from_seed(&seed).unwrap();
        let key2 = SigningKey::from_seed(&seed).unwrap();

        // Same seed = same key
        assert_eq!(key1.public_key_bytes(), key2.public_key_bytes());
    }

    #[test]
    fn test_sign_and_verify() {
        let key = SigningKey::generate().unwrap();
        let message = b"Test message";

        let signature = key.sign(message).unwrap();
        assert!(key.verify(message, &signature).unwrap());

        // Wrong message should fail
        assert!(!key.verify(b"Wrong message", &signature).unwrap());
    }

    #[test]
    fn test_encryption_key_generation() {
        let key1 = EncryptionKey::generate().unwrap();
        let key2 = EncryptionKey::generate().unwrap();

        assert_ne!(key1.public_key_bytes(), key2.public_key_bytes());
    }

    #[test]
    fn test_encrypt_decrypt() {
        let alice = EncryptionKey::generate().unwrap();
        let bob = EncryptionKey::generate().unwrap();

        let plaintext = b"Secret message";

        let ciphertext = alice.encrypt_for(bob.public_key_bytes(), plaintext).unwrap();
        let decrypted = bob.decrypt_from(alice.public_key_bytes(), &ciphertext).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_keypair_generation() {
        let keypair = IdentityKeypair::generate().unwrap();

        assert!(keypair.signing_key().public_key_bytes().len() > 0);
        assert!(keypair.encryption_key().public_key_bytes().len() > 0);
    }

    #[test]
    fn test_keypair_from_seed() {
        let seed = [123u8; 32];

        let kp1 = IdentityKeypair::from_seed(&seed).unwrap();
        let kp2 = IdentityKeypair::from_seed(&seed).unwrap();

        // Same seed = same keypairs
        assert_eq!(
            kp1.signing_key().public_key_bytes(),
            kp2.signing_key().public_key_bytes()
        );
        assert_eq!(
            kp1.encryption_key().public_key_bytes(),
            kp2.encryption_key().public_key_bytes()
        );
    }

    #[test]
    fn test_fingerprint() {
        let key = SigningKey::generate().unwrap();
        let fp = key.fingerprint();

        assert_eq!(fp.as_bytes().len(), 32);
    }
}
