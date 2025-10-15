// Flutter bridge API
// This file defines the Rust functions callable from Flutter

use crate::crypto::{derive_key, encrypt_data, decrypt_data, generate_random_bytes, generate_salt, hash_blake3};
use crate::veilid_manager::VeilidManager;
use std::sync::Arc;
use tokio::sync::RwLock;

// Global Veilid manager instance
lazy_static::lazy_static! {
    static ref VEILID: Arc<RwLock<VeilidManager>> = Arc::new(RwLock::new(VeilidManager::new()));
}

/// Initialize the Underground Railroad system
pub async fn initialize_underground_railroad(config_dir: String) -> Result<bool, String> {
    let manager = VEILID.read().await;
    manager.initialize(config_dir).await.map_err(|e| e.to_string())?;
    Ok(true)
}

/// Shutdown the system
pub async fn shutdown_underground_railroad() -> Result<bool, String> {
    let manager = VEILID.read().await;
    manager.shutdown().await.map_err(|e| e.to_string())?;
    Ok(true)
}

/// Check if system is initialized
pub async fn is_initialized() -> Result<bool, String> {
    let manager = VEILID.read().await;
    Ok(manager.is_initialized().await)
}

/// Create a new Veilid identity (keypair)
pub async fn create_veilid_identity() -> Result<VeilidIdentityData, String> {
    let manager = VEILID.read().await;
    manager.create_identity().await.map_err(|e| e.to_string())
}

/// Create a private route for receiving messages
pub async fn create_private_route() -> Result<String, String> {
    let manager = VEILID.read().await;
    manager.create_private_route().await.map_err(|e| e.to_string())
}

/// Store encrypted data in DHT
pub async fn dht_set(key: String, value: Vec<u8>) -> Result<bool, String> {
    let manager = VEILID.read().await;
    manager.dht_set(&key, value).await.map_err(|e| e.to_string())?;
    Ok(true)
}

/// Retrieve encrypted data from DHT
pub async fn dht_get(key: String) -> Result<Option<Vec<u8>>, String> {
    let manager = VEILID.read().await;
    manager.dht_get(&key).await.map_err(|e| e.to_string())
}

/// Send encrypted message via private route
pub async fn send_message_via_route(
    route: String,
    encrypted_message: Vec<u8>,
) -> Result<bool, String> {
    let manager = VEILID.read().await;
    manager
        .send_via_private_route(&route, encrypted_message)
        .await
        .map_err(|e| e.to_string())?;
    Ok(true)
}

/// Derive encryption key from password and salt
pub async fn derive_encryption_key(password: String, salt: Vec<u8>) -> Result<Vec<u8>, String> {
    let key = derive_key(&password, &salt).map_err(|e| e.to_string())?;
    Ok(key.as_slice().to_vec())
}

/// Generate random salt for key derivation
pub async fn generate_key_salt() -> Result<Vec<u8>, String> {
    Ok(generate_salt().to_vec())
}

/// Generate random bytes
pub async fn generate_secure_random(length: usize) -> Result<Vec<u8>, String> {
    Ok(generate_random_bytes(length))
}

/// Encrypt data with ChaCha20-Poly1305
pub async fn encrypt_bytes(key: Vec<u8>, plaintext: Vec<u8>) -> Result<Vec<u8>, String> {
    encrypt_data(&key, &plaintext).map_err(|e| e.to_string())
}

/// Decrypt data with ChaCha20-Poly1305
pub async fn decrypt_bytes(key: Vec<u8>, ciphertext: Vec<u8>) -> Result<Vec<u8>, String> {
    decrypt_data(&key, &ciphertext).map_err(|e| e.to_string())
}

/// Hash data with Blake3
pub async fn hash_data(data: Vec<u8>) -> Result<Vec<u8>, String> {
    Ok(hash_blake3(&data).to_vec())
}

/// Simple health check
pub async fn health_check() -> Result<String, String> {
    Ok("Underground Railroad Rust Core: OK".to_string())
}

/// Veilid identity data for bridge
#[derive(Debug, Clone)]
pub struct VeilidIdentityData {
    pub public_key: String,
    pub secret_key: String,
    pub dht_key: String,
    pub route: String,
}
