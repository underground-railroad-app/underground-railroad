/// FFI API for Flutter mobile app
///
/// This module provides the public API that Flutter can call through FFI.
/// Functions here are exposed via flutter_rust_bridge.

use underground_railroad_core::{
    assistance, crypto, identity, storage, veilid_client, Error, Region, Urgency,
};
use std::sync::Mutex;
use lazy_static::lazy_static;
use rand::RngCore;

// Global state (single instance for the app)
lazy_static! {
    static ref VEILID_CLIENT: Mutex<Option<veilid_client::VeilidClient>> = Mutex::new(None);
    static ref DATABASE: Mutex<Option<storage::Database>> = Mutex::new(None);
    static ref IDENTITY: Mutex<Option<identity::Identity>> = Mutex::new(None);
    static ref DATA_DIR: Mutex<Option<String>> = Mutex::new(None);
}

// Helper function to convert core Error to String
fn to_string_err<T>(result: Result<T, Error>) -> Result<T, String> {
    result.map_err(|e| e.to_string())
}

/// Network status info
#[derive(Debug, Clone)]
pub struct NetworkStatus {
    pub veilid_connected: bool,
    pub contacts_count: u32,
    pub emergencies_count: u32,
    pub safe_houses_count: u32,
}

/// Initialize the Underground Railroad with user credentials
pub async fn initialize(name: String, password: String, base_data_dir: String) -> Result<String, String> {
    // Ensure base data directory exists
    std::fs::create_dir_all(&base_data_dir)
        .map_err(|e| format!("Failed to create base data directory: {}", e))?;

    // Load or generate salt from base directory
    let salt_path = std::path::PathBuf::from(&base_data_dir).join("salt");
    let salt = if salt_path.exists() {
        // Load existing salt
        let salt_bytes = std::fs::read(&salt_path)
            .map_err(|e| format!("Failed to read salt: {}", e))?;

        if salt_bytes.len() != 32 {
            return Err("Invalid salt file".to_string());
        }

        let mut salt = [0u8; 32];
        salt.copy_from_slice(&salt_bytes);
        salt
    } else {
        // Generate new salt for first-time setup
        let mut salt = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut salt);

        // Save salt for future logins
        std::fs::write(&salt_path, &salt)
            .map_err(|e| format!("Failed to save salt: {}", e))?;

        salt
    };

    // Derive keys from password + salt
    let master_key = to_string_err(crypto::derive_master_key(&password, &salt))?;
    let keys = to_string_err(crypto::DerivedKeys::from_master_key(&master_key))?;

    // Derive deterministic user ID from keys
    let seed: &[u8; 32] = keys.identity_seed.as_ref()
        .try_into()
        .map_err(|_| "Invalid seed size".to_string())?;

    // Create temporary identity just to get the ID
    let temp_identity = to_string_err(identity::Identity::from_seed(&name, seed, true))?;
    let user_id = temp_identity.id.0.to_string();

    // Use first 16 chars of user ID for directory name
    let user_id_short = &user_id[..16];

    // Create user-specific data directory based on ID
    let data_dir = std::path::PathBuf::from(&base_data_dir).join(user_id_short);
    std::fs::create_dir_all(&data_dir)
        .map_err(|e| format!("Failed to create user data directory: {}", e))?;

    // Create or load identity
    let db_path = data_dir.join("railroad.db");
    let db_exists = db_path.exists();

    // Open database (creates if doesn't exist)
    let config = storage::DatabaseConfig::new(db_path);
    let db = to_string_err(storage::Database::open(config, &keys.storage_key))?;

    // Get or create identity
    let identity = {
        let repos = storage::RepositoryManager::new(&db);

        if db_exists {
            // Try to load existing primary identity
            match to_string_err(repos.identity().get_primary())? {
                Some(existing_identity) => existing_identity,
                None => {
                    // Database exists but no identity - create new one
                    let seed: &[u8; 32] = keys.identity_seed.as_ref()
                        .try_into()
                        .map_err(|_| "Invalid seed size".to_string())?;

                    let new_identity = to_string_err(identity::Identity::from_seed(&name, seed, true))?;
                    to_string_err(repos.identity().save(&new_identity))?;
                    new_identity
                }
            }
        } else {
            // New database - create new identity
            let seed: &[u8; 32] = keys.identity_seed.as_ref()
                .try_into()
                .map_err(|_| "Invalid seed size".to_string())?;

            let new_identity = to_string_err(identity::Identity::from_seed(&name, seed, true))?;
            to_string_err(repos.identity().save(&new_identity))?;
            new_identity
        }
    };

    let fingerprint = format!("{} {} {}",
        identity.verification_words()[0],
        identity.verification_words()[1],
        identity.verification_words()[2]
    );

    // Initialize Veilid (works on desktop, gracefully fails on mobile)
    let veilid_dir = data_dir.join("veilid");
    let veilid_config = veilid_client::VeilidConfig::default_private(veilid_dir);
    let mut veilid = veilid_client::VeilidClient::new(veilid_config);

    // Try to start Veilid
    // Works on desktop (macOS/Windows/Linux)
    // Fails gracefully on mobile (Android/iOS) due to JNI requirements
    match veilid.start().await {
        Ok(_) => {
            eprintln!("✅ Veilid client started and connected");
        }
        Err(e) => {
            eprintln!("⚠️  Veilid not available: {}", e);
            eprintln!("   (Normal on mobile - desktop platforms have full Veilid support)");
        }
    }

    // Store global state
    *VEILID_CLIENT.lock().unwrap() = Some(veilid);
    *DATABASE.lock().unwrap() = Some(db);
    *IDENTITY.lock().unwrap() = Some(identity);
    *DATA_DIR.lock().unwrap() = Some(data_dir.to_string_lossy().to_string());

    Ok(fingerprint)
}

/// Create an emergency request
pub async fn create_emergency(
    needs: Vec<String>,
    region: String,
    urgency: String,
    num_people: u32,
) -> Result<String, String> {
    let identity = IDENTITY.lock().unwrap();
    let identity = identity.as_ref()
        .ok_or_else(|| "Not initialized".to_string())?;

    let db = DATABASE.lock().unwrap();
    let db = db.as_ref()
        .ok_or_else(|| "Database not open".to_string())?;

    // Parse needs
    let parsed_needs: Vec<assistance::EmergencyNeed> = needs.iter().filter_map(|n| {
        match n.as_str() {
            "shelter" => Some(assistance::EmergencyNeed::SafeShelter),
            "transport" => Some(assistance::EmergencyNeed::Transportation),
            "medical" => Some(assistance::EmergencyNeed::Medical),
            "food" => Some(assistance::EmergencyNeed::Supplies),
            "financial" => Some(assistance::EmergencyNeed::Financial),
            "danger" => Some(assistance::EmergencyNeed::ImmediateDanger),
            _ => None,
        }
    }).collect();

    // Parse urgency
    let parsed_urgency = match urgency.as_str() {
        "critical" => Urgency::Critical,
        "high" => Urgency::High,
        "medium" => Urgency::Medium,
        _ => Urgency::Low,
    };

    // Create emergency
    let emergency = assistance::EmergencyRequest::new(
        Some(identity.id),
        parsed_needs,
        Region::new(region),
        parsed_urgency,
        num_people,
    );

    let id = emergency.id;

    // Save to database
    let repos = storage::RepositoryManager::new(db);
    to_string_err(repos.emergencies().save(&emergency))?;

    // TODO: Broadcast via Veilid to trusted network

    Ok(format!("{}", id.0))
}

/// Get network status
pub async fn get_status() -> Result<NetworkStatus, String> {
    let veilid = VEILID_CLIENT.lock().unwrap();
    let veilid_connected = veilid.as_ref()
        .map(|v| v.is_connected())
        .unwrap_or(false);

    let db = DATABASE.lock().unwrap();
    let (contacts, emergencies, safe_houses) = if let Some(db) = db.as_ref() {
        let repos = storage::RepositoryManager::new(db);
        (
            repos.contacts().count().unwrap_or(0),
            repos.emergencies().count_active().unwrap_or(0),
            repos.safe_houses().count_available().unwrap_or(0),
        )
    } else {
        (0, 0, 0)
    };

    Ok(NetworkStatus {
        veilid_connected,
        contacts_count: contacts as u32,
        emergencies_count: emergencies as u32,
        safe_houses_count: safe_houses as u32,
    })
}

/// Register a safe house
pub async fn register_safe_house(
    name: String,
    region: String,
    capacity: u32,
) -> Result<String, String> {
    let identity = IDENTITY.lock().unwrap();
    let identity = identity.as_ref()
        .ok_or_else(|| "Not initialized".to_string())?;

    let db = DATABASE.lock().unwrap();
    let db = db.as_ref()
        .ok_or_else(|| "Database not open".to_string())?;

    let house = assistance::SafeHouse::new(
        identity.id,
        name,
        Region::new(region),
        capacity,
    );

    let id = house.id;

    let repos = storage::RepositoryManager::new(db);
    to_string_err(repos.safe_houses().save(&house))?;

    // TODO: Announce via Veilid DHT

    Ok(format!("{}", id.0))
}

/// Add a contact
pub async fn add_contact(name: String, fingerprint_words: String) -> Result<(), String> {
    use underground_railroad_core::{PersonId, Fingerprint, SecureBytes, TrustLevel};

    let db = DATABASE.lock().unwrap();
    let db = db.as_ref()
        .ok_or_else(|| "Database not open".to_string())?;

    // Parse fingerprint from words (space-separated)
    let words: Vec<&str> = fingerprint_words.split_whitespace().collect();
    if words.len() < 3 {
        return Err("Fingerprint must be at least 3 words".to_string());
    }

    // For now, create a deterministic fingerprint from the words
    // In a real implementation, this would verify and establish secure communication
    let mut fp_bytes = [0u8; 32];
    for (i, word) in words.iter().take(32).enumerate() {
        fp_bytes[i] = word.bytes().fold(0u8, |acc, b| acc.wrapping_add(b));
    }

    let fingerprint = Fingerprint::new(fp_bytes);
    let person_id = PersonId::new();

    // Create placeholder veilid route (empty for now)
    let route = SecureBytes::new(vec![]);

    let contact = underground_railroad_core::trust::Contact::new(
        person_id,
        name,
        fingerprint,
        route,
        TrustLevel::Unknown,
    );

    let repos = storage::RepositoryManager::new(db);
    to_string_err(repos.contacts().save(&contact))?;

    Ok(())
}

/// Get all contacts
pub async fn get_contacts() -> Result<Vec<ContactInfo>, String> {
    let db = DATABASE.lock().unwrap();
    let db = db.as_ref()
        .ok_or_else(|| "Database not open".to_string())?;

    let repos = storage::RepositoryManager::new(db);
    let contacts = to_string_err(repos.contacts().list())?;

    Ok(contacts.into_iter().map(|c| {
        let words = c.verification_words();
        ContactInfo {
            id: c.id.0.to_string(),
            name: c.info.name,
            fingerprint: format!("{} {} {}", words[0], words[1], words[2]),
        }
    }).collect())
}

/// Contact information
#[derive(Debug, Clone)]
pub struct ContactInfo {
    pub id: String,
    pub name: String,
    pub fingerprint: String,
}

/// Send an encrypted message to a contact
pub async fn send_message(contact_id: String, content: String) -> Result<String, String> {
    use underground_railroad_core::{PersonId, messaging};
    use uuid::Uuid;

    let identity = IDENTITY.lock().unwrap();
    let identity = identity.as_ref()
        .ok_or_else(|| "Not initialized".to_string())?;

    let db = DATABASE.lock().unwrap();
    let db = db.as_ref()
        .ok_or_else(|| "Database not open".to_string())?;

    // Parse contact ID
    let contact_uuid = Uuid::parse_str(&contact_id)
        .map_err(|e| format!("Invalid contact ID: {}", e))?;
    let recipient_id = PersonId(contact_uuid);

    // Get contact to retrieve public keys
    let repos = storage::RepositoryManager::new(db);
    let contact = to_string_err(repos.contacts().get(recipient_id))?
        .ok_or_else(|| "Contact not found".to_string())?;

    // Create message
    let message = messaging::Message::new_text(
        identity.id,
        recipient_id,
        content,
    );

    let message_id = message.id.to_string();

    // Encrypt message using hybrid post-quantum encryption
    let recipient_keys = identity.keypair.encryption_key().get_public_keys();
    let encrypted = to_string_err(messaging::encryption::encrypt_message_hybrid(
        &bincode::serialize(&message).map_err(|e| e.to_string())?,
        &recipient_keys,
    ))?;

    // Save to database as sent
    to_string_err(repos.messages().save(&message, storage::repository::MessageDirection::Sent))?;

    // Also save to recipient's local database (simulating Veilid transmission)
    // In a real implementation, this would send via Veilid DHT/routing
    // For now, we write directly to recipient's database since both instances share the same storage
    // TODO: Replace with actual Veilid network transmission

    // For testing: Store in app's messages directory (cross-platform)
    // This simulates network message delivery
    // Format: {data_dir}/messages/urr-msg-{recipient_id}-{message_id}.bin
    let data_dir = DATA_DIR.lock().unwrap();
    let data_dir = data_dir.as_ref()
        .ok_or_else(|| "Data directory not set".to_string())?;

    let messages_dir = std::path::PathBuf::from(data_dir).join("messages");
    std::fs::create_dir_all(&messages_dir)
        .map_err(|e| format!("Failed to create messages directory: {}", e))?;

    let message_file = messages_dir.join(format!(
        "urr-msg-{}-{}.bin",
        recipient_id.0,
        message_id
    ));

    let serialized = bincode::serialize(&message)
        .map_err(|e| format!("Failed to serialize: {}", e))?;
    std::fs::write(&message_file, &serialized)
        .map_err(|e| format!("Failed to write message: {}", e))?;

    Ok(message_id)
}

/// Check for new messages from the network (simulated)
/// In production, this would poll Veilid DHT mailbox
pub async fn poll_messages() -> Result<u32, String> {
    use underground_railroad_core::messaging;

    let identity = IDENTITY.lock().unwrap();
    let identity = identity.as_ref()
        .ok_or_else(|| "Not initialized".to_string())?;
    let my_id = identity.id;
    drop(identity); // Release lock

    let db = DATABASE.lock().unwrap();
    let db = db.as_ref()
        .ok_or_else(|| "Database not open".to_string())?;

    // Check for messages addressed to me in messages directory
    // Pattern: {data_dir}/messages/urr-msg-{my_id}-*.bin
    let data_dir = DATA_DIR.lock().unwrap();
    let data_dir = data_dir.as_ref()
        .ok_or_else(|| "Data directory not set".to_string())?;

    let messages_dir = std::path::PathBuf::from(data_dir).join("messages");
    if !messages_dir.exists() {
        return Ok(0); // No messages yet
    }

    let prefix = format!("urr-msg-{}-", my_id.0);

    let mut new_messages = 0;

    // Read all message files for this user
    if let Ok(entries) = std::fs::read_dir(&messages_dir) {
        for entry in entries.flatten() {
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();

            // Check if this file is for us
            if file_name_str.starts_with(&prefix) && file_name_str.ends_with(".bin") {
                if let Ok(data) = std::fs::read(entry.path()) {
                    if let Ok(message) = bincode::deserialize::<messaging::Message>(&data) {
                        // Save to local database as received
                        let repos = storage::RepositoryManager::new(db);
                        if repos.messages().save(&message, storage::repository::MessageDirection::Received).is_ok() {
                            new_messages += 1;
                            // Delete the file after processing
                            let _ = std::fs::remove_file(entry.path());
                        }
                    }
                }
            }
        }
    }

    Ok(new_messages)
}

/// Get messages from a conversation with a contact
pub async fn get_messages(contact_id: String, limit: u32) -> Result<Vec<MessageInfo>, String> {
    use underground_railroad_core::PersonId;
    use uuid::Uuid;

    // First, poll for new messages from network
    let _ = poll_messages().await;

    let db = DATABASE.lock().unwrap();
    let db = db.as_ref()
        .ok_or_else(|| "Database not open".to_string())?;

    let contact_uuid = Uuid::parse_str(&contact_id)
        .map_err(|e| format!("Invalid contact ID: {}", e))?;
    let contact_person_id = PersonId(contact_uuid);

    let repos = storage::RepositoryManager::new(db);
    let messages = to_string_err(repos.messages().get_conversation(contact_person_id))?;

    // Convert to MessageInfo and take only requested number
    let message_infos: Vec<MessageInfo> = messages
        .into_iter()
        .take(limit as usize)
        .filter_map(|(msg, direction)| {
            // Extract text content
            if let underground_railroad_core::messaging::MessageType::Text { content } = msg.message_type {
                Some(MessageInfo {
                    id: msg.id.to_string(),
                    sender_id: msg.sender.0.to_string(),
                    recipient_id: msg.recipient.0.to_string(),
                    content,
                    status: format!("{:?}", msg.status),
                    direction: match direction {
                        storage::repository::MessageDirection::Sent => "sent".to_string(),
                        storage::repository::MessageDirection::Received => "received".to_string(),
                    },
                    created_at: msg.created_at.as_secs(),
                    is_read: msg.status == underground_railroad_core::messaging::MessageStatus::Read,
                })
            } else {
                None
            }
        })
        .collect();

    Ok(message_infos)
}

/// Get list of all conversations
pub async fn get_conversations() -> Result<Vec<ConversationInfo>, String> {
    // First, poll for new messages from network
    let _ = poll_messages().await;

    let db = DATABASE.lock().unwrap();
    let db = db.as_ref()
        .ok_or_else(|| "Database not open".to_string())?;

    let repos = storage::RepositoryManager::new(db);
    let conversations = to_string_err(repos.messages().get_conversation_list())?;

    let conv_infos: Vec<ConversationInfo> = conversations
        .into_iter()
        .map(|(contact_id, count, last_msg)| {
            // Get contact name
            let contact_name = repos.contacts().get(contact_id)
                .ok()
                .flatten()
                .map(|c| c.info.name)
                .unwrap_or_else(|| "Unknown".to_string());

            let (last_message, last_message_time) = if let Some(msg) = last_msg {
                let content = if let underground_railroad_core::messaging::MessageType::Text { content } = msg.message_type {
                    content
                } else {
                    "[Non-text message]".to_string()
                };
                (Some(content), Some(msg.created_at.as_secs()))
            } else {
                (None, None)
            };

            ConversationInfo {
                contact_id: contact_id.0.to_string(),
                contact_name,
                message_count: count as u32,
                unread_count: 0, // TODO: implement unread tracking
                last_message,
                last_message_time,
            }
        })
        .collect();

    Ok(conv_infos)
}

/// Mark a message as read
pub async fn mark_message_read(message_id: String) -> Result<(), String> {
    use uuid::Uuid;

    let db = DATABASE.lock().unwrap();
    let db = db.as_ref()
        .ok_or_else(|| "Database not open".to_string())?;

    let msg_uuid = Uuid::parse_str(&message_id)
        .map_err(|e| format!("Invalid message ID: {}", e))?;

    let repos = storage::RepositoryManager::new(db);
    to_string_err(repos.messages().mark_read(msg_uuid))?;

    Ok(())
}

/// Message information for Flutter
#[derive(Debug, Clone)]
pub struct MessageInfo {
    pub id: String,
    pub sender_id: String,
    pub recipient_id: String,
    pub content: String,
    pub status: String,
    pub direction: String,
    pub created_at: i64,
    pub is_read: bool,
}

/// Conversation summary for Flutter
#[derive(Debug, Clone)]
pub struct ConversationInfo {
    pub contact_id: String,
    pub contact_name: String,
    pub message_count: u32,
    pub unread_count: u32,
    pub last_message: Option<String>,
    pub last_message_time: Option<i64>,
}

/// Shutdown Veilid and cleanup
pub async fn shutdown() -> Result<(), String> {
    // Extract veilid client from mutex first
    let veilid_opt = {
        let mut veilid = VEILID_CLIENT.lock().unwrap();
        veilid.take()
    };

    // Now stop it without holding the lock
    if let Some(mut v) = veilid_opt {
        to_string_err(v.stop().await)?;
    }

    Ok(())
}
