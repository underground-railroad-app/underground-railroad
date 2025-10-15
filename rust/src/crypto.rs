use crate::error::{Result, UndergroundError};
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2, Params,
};
use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng},
    ChaCha20Poly1305, Nonce,
};
use rand::RngCore;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Secure memory buffer that zeros on drop
#[derive(ZeroizeOnDrop)]
pub struct SecureBuffer(Vec<u8>);

impl SecureBuffer {
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Key derivation using Argon2id
pub fn derive_key(password: &str, salt: &[u8]) -> Result<SecureBuffer> {
    let params = Params::new(65536, 3, 4, Some(32))
        .map_err(|e| UndergroundError::Crypto(e.to_string()))?;

    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);

    let salt_string = SaltString::encode_b64(salt)
        .map_err(|e| UndergroundError::Crypto(e.to_string()))?;

    let hash = argon2
        .hash_password(password.as_bytes(), &salt_string)
        .map_err(|e| UndergroundError::Crypto(e.to_string()))?;

    let key_bytes = hash
        .hash
        .ok_or_else(|| UndergroundError::Crypto("Hash generation failed".to_string()))?
        .as_bytes()
        .to_vec();

    Ok(SecureBuffer::new(key_bytes))
}

/// Generate cryptographically secure random bytes
pub fn generate_random_bytes(len: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; len];
    OsRng.fill_bytes(&mut bytes);
    bytes
}

/// Generate a random salt for key derivation
pub fn generate_salt() -> [u8; 32] {
    let mut salt = [0u8; 32];
    OsRng.fill_bytes(&mut salt);
    salt
}

/// Encrypt data using ChaCha20-Poly1305
pub fn encrypt_data(key: &[u8], plaintext: &[u8]) -> Result<Vec<u8>> {
    if key.len() != 32 {
        return Err(UndergroundError::Crypto("Key must be 32 bytes".to_string()));
    }

    let cipher = ChaCha20Poly1305::new_from_slice(key)
        .map_err(|e| UndergroundError::Crypto(e.to_string()))?;

    // Generate random nonce
    let nonce_bytes = generate_random_bytes(12);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| UndergroundError::Crypto(e.to_string()))?;

    // Prepend nonce to ciphertext
    let mut result = nonce_bytes;
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

/// Decrypt data using ChaCha20-Poly1305
pub fn decrypt_data(key: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>> {
    if key.len() != 32 {
        return Err(UndergroundError::Crypto("Key must be 32 bytes".to_string()));
    }

    if ciphertext.len() < 12 {
        return Err(UndergroundError::Crypto("Invalid ciphertext".to_string()));
    }

    let cipher = ChaCha20Poly1305::new_from_slice(key)
        .map_err(|e| UndergroundError::Crypto(e.to_string()))?;

    // Extract nonce and ciphertext
    let (nonce_bytes, ct) = ciphertext.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    // Decrypt
    let plaintext = cipher
        .decrypt(nonce, ct)
        .map_err(|e| UndergroundError::Crypto(e.to_string()))?;

    Ok(plaintext)
}

/// Blake3 hash
pub fn hash_blake3(data: &[u8]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(data);
    let hash = hasher.finalize();
    *hash.as_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_derivation() {
        let password = "test_password_123";
        let salt = generate_salt();

        let key1 = derive_key(password, &salt).unwrap();
        let key2 = derive_key(password, &salt).unwrap();

        assert_eq!(key1.as_slice(), key2.as_slice());
    }

    #[test]
    fn test_encrypt_decrypt() {
        let key = generate_random_bytes(32);
        let plaintext = b"Hello, Underground Railroad!";

        let ciphertext = encrypt_data(&key, plaintext).unwrap();
        let decrypted = decrypt_data(&key, &ciphertext).unwrap();

        assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_blake3_hash() {
        let data = b"test data";
        let hash1 = hash_blake3(data);
        let hash2 = hash_blake3(data);

        assert_eq!(hash1, hash2);
    }
}
