import 'package:flutter/foundation.dart';
import 'package:veilid/veilid.dart';
import 'veilid_service.dart';

/// Service for sending and receiving messages via Veilid DHT
class VeilidMessagingService {
  static final VeilidMessagingService _instance = VeilidMessagingService._internal();
  factory VeilidMessagingService() => _instance;
  VeilidMessagingService._internal();

  final VeilidService _veilidService = VeilidService();

  // Our mailbox DHT record
  DHTRecordDescriptor? _mailboxDescriptor;
  TypedKey? _mailboxKey;

  bool get hasMailbox => _mailboxDescriptor != null;

  /// Initialize mailbox for receiving messages
  Future<String?> createMailbox() async {
    if (!_veilidService.isConnected) {
      debugPrint('⚠️ Veilid not connected, cannot create mailbox');
      return null;
    }

    try {
      debugPrint('📬 Creating Veilid DHT mailbox...');

      // Get routing context with privacy
      final routingContext = await Veilid.instance.routingContext();
      final safeRoutingContext = await routingContext.withDefaultSafety();

      // Create a private DHT record for the mailbox
      // SMPL schema allows multiple writers (for people sending us messages)
      final schema = DHTSchema.smpl(
        oCnt: 1, // Owner count (us)
        members: [], // Empty member list initially - anyone can write
      );

      _mailboxDescriptor = await safeRoutingContext.createDHTRecord(schema);
      _mailboxKey = _mailboxDescriptor!.key;

      debugPrint('✅ Mailbox created with key: ${_mailboxKey!.toString()}');

      // Return the key as a string for storage
      return _mailboxKey!.toString();
    } catch (e) {
      debugPrint('❌ Failed to create mailbox: $e');
      return null;
    }
  }

  /// Load existing mailbox from key string
  Future<bool> loadMailbox(String mailboxKeyStr) async {
    if (!_veilidService.isConnected) {
      debugPrint('⚠️ Veilid not connected, cannot load mailbox');
      return false;
    }

    try {
      debugPrint('📬 Loading Veilid mailbox...');

      // Parse the key string to TypedKey
      _mailboxKey = TypedKey.fromString(mailboxKeyStr);

      // Get routing context
      final routingContext = await Veilid.instance.routingContext();
      final safeRoutingContext = await routingContext.withDefaultSafety();

      // Open the DHT record
      _mailboxDescriptor = await safeRoutingContext.openDHTRecord(_mailboxKey!);

      debugPrint('✅ Mailbox loaded successfully');
      return true;
    } catch (e) {
      debugPrint('❌ Failed to load mailbox: $e');
      return false;
    }
  }

  /// Send a message to a contact via their Veilid mailbox
  Future<bool> sendMessage(
    String recipientMailboxKey,
    String messageId,
    Uint8List encryptedMessageData,
  ) async {
    if (!_veilidService.isConnected) {
      debugPrint('⚠️ Veilid not connected, cannot send message');
      return false;
    }

    try {
      debugPrint('📤 Sending message via Veilid...');

      // Parse recipient's mailbox key
      final recipientKey = TypedKey.fromString(recipientMailboxKey);

      // Get routing context with privacy
      final routingContext = await Veilid.instance.routingContext();
      final safeRoutingContext = await routingContext.withDefaultSafety();

      // Open recipient's DHT record
      final recipientDescriptor = await safeRoutingContext.openDHTRecord(recipientKey);

      // Find an empty subkey to write to (0-49)
      bool written = false;
      for (int subkey = 0; subkey < 50; subkey++) {
        try {
          // Check if subkey is empty
          final existingValue = await safeRoutingContext.getDHTValue(recipientKey, subkey, forceRefresh: false);

          if (existingValue == null) {
            // Empty subkey - write message here
            await safeRoutingContext.setDHTValue(
              recipientKey,
              subkey,
              encryptedMessageData,
            );

            debugPrint('✅ Message sent to subkey $subkey');
            written = true;
            break;
          }
        } catch (e) {
          // Subkey might not be readable or other error, try next
          debugPrint('⚠️ Error checking subkey $subkey: $e');
          continue;
        }
      }

      // Close the recipient's record
      await safeRoutingContext.closeDHTRecord(recipientKey);

      if (!written) {
        debugPrint('❌ Mailbox full - no empty subkeys found');
        return false;
      }

      return true;
    } catch (e) {
      debugPrint('❌ Failed to send message via Veilid: $e');
      return false;
    }
  }

  /// Poll mailbox for new messages
  Future<List<Uint8List>> pollMessages() async {
    if (!_veilidService.isConnected || _mailboxDescriptor == null) {
      debugPrint('⚠️ Cannot poll messages - Veilid not connected or no mailbox');
      return [];
    }

    try {
      debugPrint('📥 Polling Veilid mailbox...');

      // Get routing context
      final routingContext = await Veilid.instance.routingContext();
      final safeRoutingContext = await routingContext.withDefaultSafety();

      final messages = <Uint8List>[];

      // Check all subkeys (0-49)
      for (int subkey = 0; subkey < 50; subkey++) {
        try {
          // Get value from subkey (force refresh to get latest)
          final valueData = await safeRoutingContext.getDHTValue(
            _mailboxKey!,
            subkey,
            forceRefresh: true,
          );

          if (valueData != null && valueData.data.isNotEmpty) {
            // Found a message
            messages.add(valueData.data);

            debugPrint('📬 Found message in subkey $subkey');

            // Clear the subkey after reading (write empty data)
            try {
              await safeRoutingContext.setDHTValue(
                _mailboxKey!,
                subkey,
                Uint8List(0), // Empty data
              );
            } catch (e) {
              debugPrint('⚠️ Failed to clear subkey $subkey: $e');
            }
          }
        } catch (e) {
          // Error reading this subkey, skip it
          debugPrint('⚠️ Error reading subkey $subkey: $e');
        }
      }

      if (messages.isNotEmpty) {
        debugPrint('✅ Retrieved ${messages.length} new messages');
      }

      return messages;
    } catch (e) {
      debugPrint('❌ Failed to poll messages: $e');
      return [];
    }
  }

  /// Close mailbox (cleanup)
  Future<void> closeMailbox() async {
    if (_mailboxDescriptor != null && _veilidService.isConnected) {
      try {
        final routingContext = await Veilid.instance.routingContext();
        final safeRoutingContext = await routingContext.withDefaultSafety();
        await safeRoutingContext.closeDHTRecord(_mailboxKey!);
        debugPrint('✅ Mailbox closed');
      } catch (e) {
        debugPrint('⚠️ Error closing mailbox: $e');
      }
    }
    _mailboxDescriptor = null;
    _mailboxKey = null;
  }
}
