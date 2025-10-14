# Veilid Messaging Implementation Status

## Overview

Veilid-based messaging has been implemented using a DHT mailbox architecture. Each user has a private DHT record that acts as their mailbox, where others can leave encrypted messages.

## Architecture

### DHT Mailbox Design
- **Mailbox**: Each user has a DHT record with SMPL schema (allows 50 different senders)
- **Subkeys**: Messages are stored as subkeys (0-49) in the DHT record
- **Encryption**: Messages are encrypted before writing to DHT
- **Delivery**: Sender writes to recipient's mailbox subkey, recipient polls and clears subkeys

### Components

#### Rust Core
- `core/src/identity/mod.rs`: Added `veilid_mailbox: Option<Vec<u8>>` field to Identity
- `core/src/veilid_client/messaging.rs`: VeilidMailbox implementation (for future desktop use)

#### FFI Layer
- `ffi/src/api.rs`:
  - `get_mailbox_key()`: Retrieves user's mailbox key from Identity
  - `set_mailbox_key(mailbox_key_hex)`: Stores mailbox key in Identity
  - File-based message relay still active as fallback

#### Flutter/Dart Layer
- `mobile/lib/services/veilid_messaging_service.dart`:
  - `createMailbox()`: Creates DHT record for receiving messages
  - `loadMailbox(key)`: Loads existing mailbox from key
  - `sendMessage(recipientMailboxKey, messageId, encryptedData)`: Sends message via DHT
  - `pollMessages()`: Retrieves and clears messages from own mailbox

- `mobile/lib/services/railroad_service.dart`:
  - Mailbox creation/loading during initialization
  - `_pollVeilidMessages()`: Polls Veilid mailbox for new messages
  - Integrated into `getMessages()` and `getConversations()`

## Current Status

### ✅ Implemented
1. DHT mailbox creation and loading
2. Mailbox key storage in Identity
3. Message polling from Veilid DHT
4. Graceful fallback when Veilid unavailable
5. File-based message relay (working fallback)

### ⚠️ Partially Implemented
1. **Message Sending via Veilid**: Infrastructure exists but needs:
   - Contact mailbox keys to be stored (currently not persisted)
   - Serialization of encrypted messages to Uint8List
   - Integration into `send_message()` flow

2. **Message Deserialization**: When polling Veilid messages:
   - Currently retrieves raw Uint8List data
   - Needs to deserialize to Message struct
   - Needs to save to database via FFI

### ❌ Not Yet Implemented
1. **Contact Mailbox Key Exchange**:
   - When adding a contact, their mailbox key needs to be exchanged
   - Could be done via QR code (include mailbox key in contact card)
   - Or via out-of-band exchange

2. **Message Encryption for Veilid**:
   - Messages need to be encrypted before sending to DHT
   - Should use contact's public key
   - Currently handled by FFI for file-based messages

3. **Veilid Message Storage**:
   - Polled Veilid messages need to be deserialized
   - Saved to database as "received" messages
   - Properly linked to sender contact

4. **Desktop Veilid Integration**:
   - Desktop currently uses Rust Veilid client
   - Mobile uses veilid-flutter plugin
   - Both paths should converge

## How It Works (Current State)

### Initialization
1. User logs in → FFI creates/loads Identity
2. Veilid starts via veilid-flutter plugin
3. Mailbox key checked: if exists, load; if not, create new
4. Mailbox key saved to Identity via FFI

### Sending Messages (Current)
1. User sends message → FFI `send_message()` called
2. Message encrypted and saved to sender's database
3. Message written to shared file directory (fallback)
4. **TODO**: Also write to recipient's Veilid mailbox (if we have their key)

### Receiving Messages (Current)
1. User opens conversation/messages screen
2. `_pollVeilidMessages()` called → polls DHT mailbox
3. Raw message data retrieved from Veilid
4. **TODO**: Deserialize and save to database
5. FFI `poll_messages()` called → picks up file-based messages
6. Messages returned to UI

## Next Steps for Full Veilid Integration

### Priority 1: Contact Mailbox Keys
```dart
// When adding contact, also store their mailbox key
Future<void> addContact(String name, String fingerprint, String mailboxKey) async {
  await _api.crateApiAddContact(name, fingerprint);
  // TODO: Store mailbox key in contact record
}
```

### Priority 2: Send via Veilid
```dart
Future<String> sendMessage(String contactId, String content) async {
  // 1. Get contact's mailbox key from database
  final contact = await _getContact(contactId);

  if (contact.mailboxKey != null && _veilidService.isConnected) {
    // 2. Create and encrypt message
    final message = await _api.crateApiCreateMessage(contactId, content);
    final serialized = serializeMessage(message);

    // 3. Send via Veilid
    await _messagingService.sendMessage(
      contact.mailboxKey!,
      message.id,
      serialized,
    );
  }

  // 4. Also save/send via file-based system (fallback)
  return await _api.crateApiSendMessage(contactId, content);
}
```

### Priority 3: Deserialize Veilid Messages
```dart
Future<void> _pollVeilidMessages() async {
  final encryptedMessages = await _messagingService.pollMessages();

  for (final encrypted in encryptedMessages) {
    // Deserialize from Uint8List to Message
    final message = deserializeMessage(encrypted);

    // Save to database via FFI
    await _api.crateApiSaveReceivedMessage(message);
  }
}
```

## Testing

### File-Based Messaging (Working Now)
1. Create two users with different credentials
2. Add each other as contacts
3. Send messages between them
4. Messages flow via shared file directory
5. Both users can see messages in conversations

### Veilid Messaging (Partial)
1. Veilid connects on app start (if network available)
2. Mailbox created/loaded successfully
3. Polling mailbox works (retrieves empty data currently)
4. Sending via Veilid requires contact mailbox keys

## Security Considerations

- ✅ Messages encrypted before storage
- ✅ DHT mailbox is private (SMPL schema)
- ✅ Mailbox key stored securely in encrypted database
- ⚠️ Contact mailbox keys need secure storage
- ⚠️ Message serialization format needs review
- ⚠️ DHT mailbox can fill up (50 message limit) - need cleanup strategy

## Performance Notes

- DHT operations add latency vs file-based
- Mailbox polling on every message load may be excessive
- Consider background polling service
- Subkey iteration (0-49) is inefficient - could track last read position
