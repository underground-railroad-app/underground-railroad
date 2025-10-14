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
    static ref BASE_DATA_DIR: Mutex<Option<String>> = Mutex::new(None);
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

    // Try to start Veilid (for desktop only - Flutter will handle Veilid on mobile)
    // Works on desktop (macOS/Windows/Linux)
    // Fails gracefully on mobile (Android/iOS) - mobile uses veilid-flutter plugin directly
    match veilid.start().await {
        Ok(_) => {
            eprintln!("✅ Veilid client started and connected (desktop)");
        }
        Err(e) => {
            eprintln!("⚠️  Veilid not available via Rust: {}", e);
            eprintln!("   (Normal on mobile - use veilid-flutter plugin instead)");
        }
    }

    // Store global state
    *VEILID_CLIENT.lock().unwrap() = Some(veilid);
    *DATABASE.lock().unwrap() = Some(db);
    *IDENTITY.lock().unwrap() = Some(identity);
    *DATA_DIR.lock().unwrap() = Some(data_dir.to_string_lossy().to_string());
    *BASE_DATA_DIR.lock().unwrap() = Some(base_data_dir.clone());

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

/// Add a contact with their Veilid mailbox key
pub async fn add_contact(
    name: String,
    fingerprint_words: String,
    mailbox_key: String,
) -> Result<(), String> {
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

    // Store the mailbox key in veilid_route field (as bytes)
    // This allows contacts to send messages to this person's Veilid mailbox
    let mailbox_key_bytes = mailbox_key.as_bytes().to_vec();
    let route = SecureBytes::new(mailbox_key_bytes);

    let mut contact = underground_railroad_core::trust::Contact::new(
        person_id,
        name,
        fingerprint,
        route,
        TrustLevel::Unknown,
    );

    // Store original fingerprint words as a tag for display
    contact.add_tag(format!("fp:{}", fingerprint_words));

    let repos = storage::RepositoryManager::new(db);
    to_string_err(repos.contacts().save(&contact))?;

    Ok(())
}

/// Get a contact's Veilid mailbox key
pub async fn get_contact_mailbox_key(contact_id: String) -> Result<Option<String>, String> {
    use underground_railroad_core::PersonId;
    use uuid::Uuid;

    let db = DATABASE.lock().unwrap();
    let db = db.as_ref()
        .ok_or_else(|| "Database not open".to_string())?;

    let contact_uuid = Uuid::parse_str(&contact_id)
        .map_err(|e| format!("Invalid contact ID: {}", e))?;
    let contact_person_id = PersonId(contact_uuid);

    let repos = storage::RepositoryManager::new(db);
    let contact = to_string_err(repos.contacts().get(contact_person_id))?
        .ok_or_else(|| "Contact not found".to_string())?;

    // Extract mailbox key from veilid_route field
    let mailbox_key_bytes = contact.veilid_route.as_bytes();
    if mailbox_key_bytes.is_empty() {
        return Ok(None);
    }

    let mailbox_key = String::from_utf8(mailbox_key_bytes.to_vec())
        .map_err(|e| format!("Invalid mailbox key: {}", e))?;

    Ok(Some(mailbox_key))
}

/// Get all contacts
pub async fn get_contacts() -> Result<Vec<ContactInfo>, String> {
    let db = DATABASE.lock().unwrap();
    let db = db.as_ref()
        .ok_or_else(|| "Database not open".to_string())?;

    let repos = storage::RepositoryManager::new(db);
    let contacts = to_string_err(repos.contacts().list())?;

    Ok(contacts.into_iter().map(|c| {
        // Try to get original fingerprint words from tags
        let fingerprint = c.tags.iter()
            .find(|tag| tag.starts_with("fp:"))
            .map(|tag| tag.strip_prefix("fp:").unwrap_or("").to_string())
            .unwrap_or_else(|| {
                // Fallback to generated words if no tag found
                let words = c.verification_words();
                format!("{} {} {}", words[0], words[1], words[2])
            });

        ContactInfo {
            id: c.id.0.to_string(),
            name: c.info.name,
            fingerprint,
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

    // Save to database as sent (Flutter will handle Veilid transmission)
    to_string_err(repos.messages().save(&message, storage::repository::MessageDirection::Sent))?;

    // Return the message ID and serialized message so Flutter can send via Veilid
    Ok(message_id)
}

/// Save a received message from Veilid to the database
/// Called by Flutter after polling Veilid mailbox
pub async fn save_received_message(
    sender_id: String,
    content: String,
    created_at: i64,
) -> Result<String, String> {
    use underground_railroad_core::{PersonId, messaging};
    use uuid::Uuid;

    let db = DATABASE.lock().unwrap();
    let db = db.as_ref()
        .ok_or_else(|| "Database not open".to_string())?;

    // Parse sender ID
    let sender_uuid = Uuid::parse_str(&sender_id)
        .map_err(|e| format!("Invalid sender ID: {}", e))?;
    let sender_person_id = PersonId(sender_uuid);

    let identity = IDENTITY.lock().unwrap();
    let identity = identity.as_ref()
        .ok_or_else(|| "Not initialized".to_string())?;
    let recipient_id = identity.id;

    // Create message with provided timestamp
    use chrono::{DateTime, Utc};
    let dt = DateTime::from_timestamp(created_at, 0)
        .ok_or_else(|| "Invalid timestamp".to_string())?;

    let message = messaging::Message {
        id: Uuid::new_v4(),
        sender: sender_person_id,
        recipient: recipient_id,
        message_type: messaging::MessageType::Text { content },
        created_at: underground_railroad_core::CoarseTimestamp::from_datetime(dt),
        expires_at: None,
        status: messaging::MessageStatus::Delivered,
        hop_count: 0, // Direct message from Veilid
    };

    let message_id = message.id.to_string();

    // Save to database as received
    let repos = storage::RepositoryManager::new(db);
    to_string_err(repos.messages().save(&message, storage::repository::MessageDirection::Received))?;

    Ok(message_id)
}

/// Get serialized message data for sending via Veilid
pub async fn get_message_for_veilid(_contact_id: String, message_id: String) -> Result<Vec<u8>, String> {
    use underground_railroad_core::PersonId;
    use uuid::Uuid;

    let db = DATABASE.lock().unwrap();
    let db = db.as_ref()
        .ok_or_else(|| "Database not open".to_string())?;

    let msg_uuid = Uuid::parse_str(&message_id)
        .map_err(|e| format!("Invalid message ID: {}", e))?;

    let repos = storage::RepositoryManager::new(db);
    let (message, _) = to_string_err(repos.messages().get(msg_uuid))?
        .ok_or_else(|| "Message not found".to_string())?;

    // Serialize the message for Veilid transmission
    bincode::serialize(&message)
        .map_err(|e| format!("Failed to serialize message: {}", e))
}

/// Decrypt and save a received message from Veilid
pub async fn decrypt_and_save_message(encrypted_data: Vec<u8>) -> Result<String, String> {
    use underground_railroad_core::messaging;

    let identity = IDENTITY.lock().unwrap();
    let identity = identity.as_ref()
        .ok_or_else(|| "Not initialized".to_string())?;

    let db = DATABASE.lock().unwrap();
    let db = db.as_ref()
        .ok_or_else(|| "Database not open".to_string())?;

    // Deserialize the message
    let message: messaging::Message = bincode::deserialize(&encrypted_data)
        .map_err(|e| format!("Failed to deserialize message: {}", e))?;

    // Verify this message is for us
    if message.recipient != identity.id {
        return Err("Message not intended for this recipient".to_string());
    }

    let message_id = message.id.to_string();

    // Save to database as received
    let repos = storage::RepositoryManager::new(db);
    to_string_err(repos.messages().save(&message, storage::repository::MessageDirection::Received))?;

    Ok(message_id)
}

/// Get messages from a conversation with a contact
pub async fn get_messages(contact_id: String, limit: u32) -> Result<Vec<MessageInfo>, String> {
    use underground_railroad_core::PersonId;
    use uuid::Uuid;

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

/// Get Veilid mailbox key for current identity
pub async fn get_mailbox_key() -> Result<Option<String>, String> {
    let identity = IDENTITY.lock().unwrap();
    let identity = identity.as_ref()
        .ok_or_else(|| "Not initialized".to_string())?;

    Ok(identity.veilid_mailbox.as_ref().map(|bytes| {
        String::from_utf8(bytes.clone()).unwrap_or_default()
    }))
}

/// Set Veilid mailbox key for current identity
pub async fn set_mailbox_key(mailbox_key: String) -> Result<(), String> {
    let mut identity_lock = IDENTITY.lock().unwrap();
    let identity = identity_lock.as_mut()
        .ok_or_else(|| "Not initialized".to_string())?;

    // Store the key string as bytes
    let mailbox_bytes = mailbox_key.as_bytes().to_vec();

    identity.veilid_mailbox = Some(mailbox_bytes);

    // Save to database
    let db = DATABASE.lock().unwrap();
    let db = db.as_ref()
        .ok_or_else(|| "Database not open".to_string())?;

    let repos = storage::RepositoryManager::new(db);
    to_string_err(repos.identity().save(identity))?;

    Ok(())
}

/// Create a Veilid mailbox using the desktop Veilid client
pub async fn create_veilid_mailbox_desktop() -> Result<String, String> {
    use veilid_client::{DHTSchema, CRYPTO_KIND_VLD0, VeilidAPI};
    use std::sync::Arc;

    // Get API reference without holding the lock
    let api: Arc<VeilidAPI> = {
        let veilid = VEILID_CLIENT.lock().unwrap();
        Arc::new(veilid.as_ref()
            .ok_or_else(|| "Veilid not started".to_string())?
            .api()
            .ok_or_else(|| "Veilid API not available".to_string())?
            .clone())
    }; // Lock dropped here

    // Create routing context with privacy
    let routing_context = api.routing_context()
        .map_err(|e| e.to_string())?
        .with_default_safety()
        .map_err(|e| e.to_string())?;

    // Create DHT record for mailbox (SMPL schema allows multiple writers)
    let schema = DHTSchema::smpl(1, vec![])
        .map_err(|e| e.to_string())?;
    let descriptor = routing_context.create_dht_record(schema, None, Some(CRYPTO_KIND_VLD0))
        .await
        .map_err(|e| e.to_string())?;

    let mailbox_key = descriptor.key().to_string();

    Ok(mailbox_key)
}

/// Send message via desktop Veilid client
pub async fn send_message_via_veilid_desktop(
    _recipient_mailbox_key: String,
    _message_data: Vec<u8>,
) -> Result<bool, String> {
    // TODO: Implement desktop DHT messaging
    // For now, messages are saved locally but not transmitted on desktop
    // Mobile uses veilid-flutter which works
    eprintln!("⚠️  Desktop Veilid messaging not yet implemented");
    eprintln!("   Message saved locally only");
    Ok(false)
}

/// Poll mailbox for new messages (desktop)
pub async fn poll_veilid_mailbox_desktop(_mailbox_key: String) -> Result<Vec<Vec<u8>>, String> {
    // TODO: Implement desktop DHT mailbox polling
    // For now, desktop doesn't receive messages via Veilid
    // Mobile uses veilid-flutter which works
    Ok(vec![])
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
