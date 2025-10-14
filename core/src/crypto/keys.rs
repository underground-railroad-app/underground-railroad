//! Key generation and management

use crate::Result;
use zeroize::{Zeroize, Zeroizing};

/// Master key (256-bit, derived from passphrase)
pub type MasterKey = Zeroizing<[u8; 32]>;

/// Storage key (for database encryption)
pub type StorageKey = Zeroizing<[u8; 32]>;

/// Identity key seed (for Ed25519 keypair)
pub type IdentitySeed = Zeroizing<[u8; 32]>;

/// Encryption key seed (for X25519 keypair)
pub type EncryptionSeed = Zeroizing<[u8; 32]>;

/// Derive master key from passphrase (Argon2id)
pub fn derive_master_key(passphrase: &str, salt: &[u8; 32]) -> Result<MasterKey> {
    use argon2::{Argon2, Params, PasswordHasher};
    use argon2::password_hash::{Salt, SaltString};

    // Argon2id parameters (mobile-friendly but still secure)
    let params = Params::new(
        64 * 1024,  // 64 MB memory
        3,          // 3 iterations
        4,          // 4 parallel threads
        Some(32),   // 32-byte output
    ).map_err(|e| crate::Error::Crypto(format!("Failed to create Argon2 params: {}", e)))?;

    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        params,
    );

    let salt_string = SaltString::encode_b64(salt)
        .map_err(|e| crate::Error::Crypto(format!("Failed to encode salt: {}", e)))?;

    let hash = argon2
        .hash_password(passphrase.as_bytes(), &salt_string)
        .map_err(|e| crate::Error::Crypto(format!("Failed to hash password: {}", e)))?;

    let hash_bytes = hash.hash
        .ok_or_else(|| crate::Error::Crypto("No hash output".to_string()))?;

    let mut key = [0u8; 32];
    key.copy_from_slice(&hash_bytes.as_bytes()[..32]);

    Ok(Zeroizing::new(key))
}

/// Derive keys from master key using HKDF
pub struct DerivedKeys {
    pub identity_seed: IdentitySeed,
    pub encryption_seed: EncryptionSeed,
    pub storage_key: StorageKey,
}

impl DerivedKeys {
    pub fn from_master_key(master_key: &MasterKey) -> Result<Self> {
        use hkdf::Hkdf;
        use sha2::Sha256;

        let hkdf = Hkdf::<Sha256>::new(None, master_key.as_ref());

        // Derive identity seed
        let mut identity_seed = Zeroizing::new([0u8; 32]);
        hkdf.expand(b"underground-railroad-identity-v1", &mut *identity_seed)
            .map_err(|e| crate::Error::Crypto(format!("Failed to derive identity seed: {}", e)))?;

        // Derive encryption seed
        let mut encryption_seed = Zeroizing::new([0u8; 32]);
        hkdf.expand(b"underground-railroad-encryption-v1", &mut *encryption_seed)
            .map_err(|e| crate::Error::Crypto(format!("Failed to derive encryption seed: {}", e)))?;

        // Derive storage key
        let mut storage_key = Zeroizing::new([0u8; 32]);
        hkdf.expand(b"underground-railroad-storage-v1", &mut *storage_key)
            .map_err(|e| crate::Error::Crypto(format!("Failed to derive storage key: {}", e)))?;

        Ok(Self {
            identity_seed,
            encryption_seed,
            storage_key,
        })
    }
}

impl Zeroize for DerivedKeys {
    fn zeroize(&mut self) {
        self.identity_seed.zeroize();
        self.encryption_seed.zeroize();
        self.storage_key.zeroize();
    }
}

impl Drop for DerivedKeys {
    fn drop(&mut self) {
        self.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_master_key_derivation() {
        let passphrase = "test passphrase with good entropy 12345";
        let salt = [42u8; 32];

        let key1 = derive_master_key(passphrase, &salt).unwrap();
        let key2 = derive_master_key(passphrase, &salt).unwrap();

        // Same passphrase + salt = same key
        assert_eq!(key1.as_ref(), key2.as_ref());

        // Different salt = different key
        let salt2 = [43u8; 32];
        let key3 = derive_master_key(passphrase, &salt2).unwrap();
        assert_ne!(key1.as_ref(), key3.as_ref());
    }

    #[test]
    fn test_key_derivation() {
        let master_key = Zeroizing::new([1u8; 32]);

        let keys1 = DerivedKeys::from_master_key(&master_key).unwrap();
        let keys2 = DerivedKeys::from_master_key(&master_key).unwrap();

        // Same master key = same derived keys
        assert_eq!(keys1.identity_seed.as_ref(), keys2.identity_seed.as_ref());
        assert_eq!(keys1.encryption_seed.as_ref(), keys2.encryption_seed.as_ref());
        assert_eq!(keys1.storage_key.as_ref(), keys2.storage_key.as_ref());

        // All derived keys should be different from each other
        assert_ne!(keys1.identity_seed.as_ref(), keys1.encryption_seed.as_ref());
        assert_ne!(keys1.identity_seed.as_ref(), keys1.storage_key.as_ref());
        assert_ne!(keys1.encryption_seed.as_ref(), keys1.storage_key.as_ref());
    }
}
