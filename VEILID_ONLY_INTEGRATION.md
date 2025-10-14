# Veilid-Only Messaging - Integration Guide

## ✅ What's Been Completed

### Rust FFI Layer
1. **Removed file-based fallback** - No more shared message directory
2. **Updated FFI functions**:
   - `add_contact(name, fingerprint, mailbox_key)` - Now requires mailbox key
   - `get_contact_mailbox_key(contact_id)` - Retrieves contact's mailbox key
   - `save_received_message(sender_id, content, created_at)` - Saves Veilid message to DB
   - `get_message_for_veilid(contact_id, message_id)` - Gets serialized message for sending
   - `get_mailbox_key()` / `set_mailbox_key()` - Manages user's own mailbox key

3. **Contact storage** - Mailbox keys stored in `veilid_route` field
4. **FFI bindings regenerated** - All functions available in Flutter

### Flutter Veilid Services
1. **VeilidMessagingService** (`veilid_messaging_service.dart`):
   - `createMailbox()` - Creates DHT mailbox for receiving
   - `loadMailbox(key)` - Loads existing mailbox
   - `sendMessage(recipientMailboxKey, messageId, encryptedData)` - Sends via DHT
   - `pollMessages()` - Retrieves messages from own mailbox

2. **RailroadService Integration**:
   - Mailbox creation/loading during initialization
   - `_pollVeilidMessages()` helper added

## ⚠️ What Needs To Be Completed

### 1. Update Flutter Service to Send via Veilid

**File**: `mobile/lib/services/railroad_service.dart`

```dart
/// Send an encrypted message to a contact
Future<String> sendMessage(String contactId, String content) async {
  try {
    debugPrint('Sending message to: $contactId');

    // Step 1: Create message and save to local DB (marks as "sent")
    final messageId = await _api.crateApiSendMessage(
      contactId: contactId,
      content: content,
    );

    // Step 2: Get contact's mailbox key
    final mailboxKey = await _api.crateApiGetContactMailboxKey(contactId: contactId);

    if (mailboxKey == null) {
      throw Exception('Contact does not have a mailbox key - cannot send via Veilid');
    }

    // Step 3: Check if Veilid is connected
    if (!_veilidService.isConnected) {
      throw Exception('Veilid is not connected - message saved locally but not sent');
    }

    // Step 4: Get serialized message for Veilid
    final messageData = await _api.crateApiGetMessageForVeilid(
      contactId: contactId,
      messageId: messageId,
    );

    // Step 5: Send via Veilid DHT
    final sent = await _messagingService.sendMessage(
      mailboxKey,
      messageId,
      Uint8List.fromList(messageData),
    );

    if (!sent) {
      throw Exception('Failed to send message via Veilid DHT');
    }

    debugPrint('✅ Message sent via Veilid: $messageId');
    return messageId;
  } catch (e) {
    debugPrint('❌ Failed to send message: $e');
    rethrow;
  }
}
```

### 2. Update Message Polling to Deserialize Veilid Messages

**File**: `mobile/lib/services/railroad_service.dart`

```dart
/// Poll Veilid for new messages and save them to database
Future<void> _pollVeilidMessages() async {
  if (!_veilidService.isConnected || !_messagingService.hasMailbox) {
    return;
  }

  try {
    // Poll for new messages from Veilid DHT
    final encryptedMessages = await _messagingService.pollMessages();

    if (encryptedMessages.isEmpty) {
      return;
    }

    debugPrint('📥 Retrieved ${encryptedMessages.length} messages from Veilid');

    // Deserialize and save each message
    for (final messageData in encryptedMessages) {
      try {
        // The messageData is a serialized Message struct from Rust
        // We need to deserialize it using bincode format
        // For now, we can create a simple parser or use FFI to deserialize

        // TODO: Implement proper Message deserialization
        // For now, extract basic fields and save

        // Example of what needs to be done:
        // 1. Deserialize messageData (bincode format) to get Message struct
        // 2. Extract: sender_id, content, created_at
        // 3. Call save_received_message FFI

        // Placeholder - you'll need to implement proper deserialization
        debugPrint('⚠️ Message deserialization not yet implemented');

      } catch (e) {
        debugPrint('❌ Failed to process Veilid message: $e');
      }
    }
  } catch (e) {
    debugPrint('⚠️ Failed to poll Veilid messages: $e');
  }
}
```

### 3. Update add_contact to Include Mailbox Key

**File**: `mobile/lib/services/railroad_service.dart`

```dart
/// Add a contact with their Veilid mailbox key
Future<void> addContact(String name, String fingerprint, String mailboxKey) async {
  try {
    debugPrint('Adding contact: $name with fingerprint: $fingerprint');

    // Call Rust FFI with mailbox key
    await _api.crateApiAddContact(
      name: name,
      fingerprintWords: fingerprint,
      mailboxKey: mailboxKey,  // NEW: Include mailbox key
    );

    debugPrint('✅ Contact added: $name');
  } catch (e) {
    debugPrint('❌ Failed to add contact: $e');
    rethrow;
  }
}
```

### 4. Update QR Code / Contact Card to Include Mailbox Key

**File**: Update wherever contacts are added (QR scanner, manual entry)

```dart
// When creating a contact card / QR code, include mailbox key
final myMailboxKey = await _api.crateApiGetMailboxKey();

final contactCard = {
  'name': myName,
  'fingerprint': myFingerprint,
  'mailbox_key': myMailboxKey,  // NEW: Include this
};

// When scanning a contact's QR code / adding contact
final contactData = parseQRCode(qrData);
await railroad.addContact(
  contactData['name'],
  contactData['fingerprint'],
  contactData['mailbox_key'],  // NEW: Extract this
);
```

## Message Deserialization Challenge

The main remaining technical challenge is deserializing Veilid messages. Messages are stored as `bincode`-serialized `Message` structs from Rust.

### Option 1: Add FFI Helper (Recommended)

Add a new FFI function to deserialize messages:

```rust
// In ffi/src/api.rs

/// Deserialize a message from Veilid bytes
pub async fn deserialize_veilid_message(message_data: Vec<u8>) -> Result<VeilidMessageInfo, String> {
    use underground_railroad_core::messaging;

    let message: messaging::Message = bincode::deserialize(&message_data)
        .map_err(|e| format!("Failed to deserialize message: {}", e))?;

    Ok(VeilidMessageInfo {
        sender_id: message.sender.0.to_string(),
        content: match message.message_type {
            messaging::MessageType::Text { content } => content,
            _ => "[Non-text message]".to_string(),
        },
        created_at: message.created_at.as_secs(),
    })
}

#[derive(Debug, Clone)]
pub struct VeilidMessageInfo {
    pub sender_id: String,
    pub content: String,
    pub created_at: i64,
}
```

Then in Flutter:

```dart
Future<void> _pollVeilidMessages() async {
  // ... poll messages ...

  for (final messageData in encryptedMessages) {
    try {
      // Deserialize using FFI
      final messageInfo = await _api.crateApiDeserializeVeilidMessage(
        messageData: messageData,
      );

      // Save to database
      await _api.crateApiSaveReceivedMessage(
        senderId: messageInfo.senderId,
        content: messageInfo.content,
        createdAt: messageInfo.createdAt,
      );

      debugPrint('✅ Saved Veilid message from ${messageInfo.senderId}');
    } catch (e) {
      debugPrint('❌ Failed to process message: $e');
    }
  }
}
```

### Option 2: Use Dart bincode Library

Install a Dart bincode library and mirror the Message struct in Dart. This is more complex but avoids FFI calls.

## Testing the Implementation

### Test Flow
1. **User A** starts app → Veilid connects → Mailbox created → Mailbox key saved
2. **User B** starts app → Veilid connects → Mailbox created → Mailbox key saved
3. **User A** adds User B as contact → Includes User B's mailbox key
4. **User B** adds User A as contact → Includes User A's mailbox key
5. **User A** sends message to User B:
   - Message saved to A's database as "sent"
   - Message serialized and sent to B's Veilid mailbox
   - Message appears in B's mailbox DHT subkey
6. **User B** opens conversation:
   - `_pollVeilidMessages()` called
   - Message retrieved from Veilid mailbox
   - Message deserialized
   - Message saved to B's database as "received"
   - Message displayed in UI

### Error Cases to Handle
- **Veilid not connected**: Clear error message, don't send
- **Contact has no mailbox key**: Error - cannot send
- **Mailbox full (50 messages)**: Need cleanup strategy
- **Message deserialization fails**: Skip message, log error
- **DHT write fails**: Retry logic

## Security Considerations

✅ **Removed**: File-based message relay (insecure)
✅ **Using**: Veilid DHT with anonymous routing
✅ **Encrypted**: Messages encrypted before DHT storage
⚠️ **TODO**: Implement message expiration/cleanup in mailboxes
⚠️ **TODO**: Rate limiting on message sending
⚠️ **TODO**: Verify sender identity (messages currently trust sender_id)

## Performance Notes

- DHT writes have network latency (~1-5 seconds)
- Mailbox polling on every conversation load may be excessive
  - Consider: Background polling service
  - Consider: Veilid `ValueChange` events instead of polling
- Subkey iteration (0-49) is inefficient
  - Consider: Track last read position
  - Consider: Use sparse subkeys

## Next Steps

1. Add `deserialize_veilid_message` FFI function
2. Update `sendMessage()` to use Veilid
3. Update `_pollVeilidMessages()` to deserialize and save
4. Update contact adding UI to include mailbox key
5. Regenerate FFI bindings: `~/.asdf/installs/rust/1.85.0/bin/flutter_rust_bridge_codegen generate`
6. Build and test: `cargo build --release && cd mobile && flutter run`

The infrastructure is complete - just need these final integration steps!
