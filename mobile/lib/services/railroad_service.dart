import 'package:flutter/foundation.dart';
import 'dart:io' show Directory, Platform;
import 'package:path_provider/path_provider.dart';

// Native platforms only - web intentionally excluded for security
import '../ffi/frb_generated.dart';
import '../ffi/api.dart' as api;
import 'veilid_service.dart';
import 'veilid_messaging_service.dart';

/// Service for communicating with Rust core + Veilid
class RailroadService {
  static final RailroadService _instance = RailroadService._internal();
  factory RailroadService() => _instance;
  RailroadService._internal();

  // Helper to access the FFI API (suppresses internal member warning)
  // ignore: invalid_use_of_internal_member
  RustLibApi get _api => RustLib.instance.api;

  // Veilid service for mobile networking
  final VeilidService _veilidService = VeilidService();
  final VeilidMessagingService _messagingService = VeilidMessagingService();

  bool _initialized = false;
  String? _fingerprint;

  bool get isInitialized => _initialized;
  String? get fingerprint => _fingerprint;

  /// Get shareable identity (fingerprint + mailbox key)
  Future<String?> getShareableIdentity() async {
    if (!_initialized || _fingerprint == null) {
      return null;
    }

    // Ensure mailbox exists
    await _ensureMailbox();

    // Get mailbox key
    final mailboxKey = await _api.crateApiGetMailboxKey();

    if (mailboxKey == null || mailboxKey.isEmpty) {
      // Return just fingerprint if no mailbox
      return _fingerprint;
    }

    // Combine fingerprint and mailbox key with | delimiter
    return '$_fingerprint|$mailboxKey';
  }

  /// Ensure mailbox exists (create if needed) - works on all platforms
  Future<bool> _ensureMailbox() async {
    try {
      // Check if we have a saved mailbox key
      final existingMailboxKey = await _api.crateApiGetMailboxKey();

      if (existingMailboxKey != null && existingMailboxKey.isNotEmpty) {
        // Mailbox already exists
        if (Platform.isAndroid || Platform.isIOS) {
          // Load on mobile if not already loaded
          if (!_messagingService.hasMailbox && _veilidService.isConnected) {
            debugPrint('📬 Loading existing mailbox (mobile)...');
            final loaded = await _messagingService.loadMailbox(existingMailboxKey);
            if (loaded) {
              debugPrint('✅ Mailbox loaded');
            }
          }
        }
        return true; // Desktop mailbox always available via FFI
      }

      // Need to create new mailbox
      String? mailboxKey;

      if (Platform.isAndroid || Platform.isIOS) {
        // Mobile: Use veilid-flutter
        if (!_veilidService.isConnected) {
          debugPrint('⚠️ Cannot create mailbox: Veilid not connected (mobile)');
          return false;
        }

        debugPrint('📬 Creating new mailbox (mobile)...');
        mailboxKey = await _messagingService.createMailbox();
      } else {
        // Desktop: Use Rust FFI
        debugPrint('📬 Creating new mailbox (desktop)...');
        mailboxKey = await _api.crateApiCreateVeilidMailboxDesktop();
      }

      if (mailboxKey != null && mailboxKey.isNotEmpty) {
        await _api.crateApiSetMailboxKey(mailboxKey: mailboxKey);
        debugPrint('✅ Mailbox created: $mailboxKey');
        return true;
      }

      return false;
    } catch (e) {
      debugPrint('❌ Mailbox creation failed: $e');
      return false;
    }
  }

  /// Initialize the Underground Railroad
  /// This creates identity, starts Veilid, connects to network
  Future<void> initialize(String name, String password) async {
    try {
      // Get base data directory (Rust FFI will create user-specific subdirectory based on user ID)
      final String baseDataDir;

      if (Platform.isMacOS || Platform.isLinux) {
        // Desktop: Use ~/.underground-railroad
        final home = Platform.environment['HOME'] ?? Platform.environment['USERPROFILE'];
        if (home == null) {
          throw Exception('Cannot determine home directory');
        }
        baseDataDir = '$home/.underground-railroad';
      } else {
        // Mobile (iOS/Android): Use app documents directory
        final Directory appDir = await getApplicationDocumentsDirectory();
        baseDataDir = '${appDir.path}/underground-railroad';
      }

      // Ensure base directory exists (Rust will create user-specific subdirectory)
      await Directory(baseDataDir).create(recursive: true);

      debugPrint('Initializing Underground Railroad...');
      debugPrint('Base data dir: $baseDataDir');
      debugPrint('Name: $name');
      debugPrint('(User-specific directory will be created based on user ID)');

      // Call Rust FFI (it will create subdirectory based on derived user ID)
      final fingerprint = await _api.crateApiInitialize(
        name: name,
        password: password,
        baseDataDir: baseDataDir,
      );

      _fingerprint = fingerprint;
      _initialized = true;

      debugPrint('✅ Initialized! Fingerprint: $_fingerprint');

      // Initialize Veilid on mobile only (desktop uses Rust-side Veilid)
      if (Platform.isAndroid || Platform.isIOS) {
        await _veilidService.initialize('UndergroundRailroad');
      } else {
        debugPrint('✅ Desktop platform: Using Rust-side Veilid (already started)');
      }

      // Create or load Veilid mailbox if connected (mobile only)
      if ((Platform.isAndroid || Platform.isIOS) && _veilidService.isConnected) {
        try {
          // Check if we already have a mailbox key
          final existingMailboxKey = await _api.crateApiGetMailboxKey();

          if (existingMailboxKey != null) {
            // Load existing mailbox
            debugPrint('📬 Loading existing Veilid mailbox...');
            final loaded = await _messagingService.loadMailbox(existingMailboxKey);
            if (loaded) {
              debugPrint('✅ Mailbox loaded successfully');
            }
          } else {
            // Create new mailbox
            debugPrint('📬 Creating new Veilid mailbox...');
            final mailboxKey = await _messagingService.createMailbox();
            if (mailboxKey != null) {
              // Save mailbox key to identity
              await _api.crateApiSetMailboxKey(mailboxKey: mailboxKey);
              debugPrint('✅ Mailbox created and saved');
            }
          }
        } catch (e) {
          debugPrint('⚠️ Mailbox setup failed: $e');
          debugPrint('   Messages will use file-based fallback');
        }
      }

    } catch (e) {
      debugPrint('❌ Initialization failed: $e');
      rethrow;
    }
  }

  /// Create emergency request
  Future<String> createEmergency({
    required List<String> needs,
    required String region,
    required String urgency,
    required int numPeople,
  }) async {
    try {
      debugPrint('Creating emergency: $needs, $numPeople people, $urgency');

      // Call Rust FFI
      final emergencyId = await _api.crateApiCreateEmergency(
        needs: needs,
        region: region,
        urgency: urgency,
        numPeople: numPeople,
      );

      debugPrint('✅ Emergency created: $emergencyId');
      debugPrint('🔄 Broadcasting to Veilid network...');

      return emergencyId;
    } catch (e) {
      debugPrint('❌ Emergency creation failed: $e');
      rethrow;
    }
  }

  /// Register safe house
  Future<String> registerSafeHouse({
    required String name,
    required String region,
    required int capacity,
  }) async {
    try {
      debugPrint('Registering safe house: $name in $region, capacity: $capacity');

      // Call Rust FFI
      final houseId = await _api.crateApiRegisterSafeHouse(
        name: name,
        region: region,
        capacity: capacity,
      );

      debugPrint('✅ Safe house registered: $houseId');
      debugPrint('📡 Announcing to Veilid DHT...');

      return houseId;
    } catch (e) {
      debugPrint('❌ Safe house registration failed: $e');
      rethrow;
    }
  }

  /// Get network status
  Future<NetworkStatus> getStatus() async {
    try {
      // Call Rust FFI for database counts
      final status = await _api.crateApiGetStatus();

      // On mobile: Use Flutter's VeilidService (veilid-flutter plugin)
      // On desktop: Use Rust FFI status (Rust manages Veilid directly)
      final veilidConnected = Platform.isAndroid || Platform.isIOS
          ? _veilidService.isConnected
          : status.veilidConnected;

      return NetworkStatus(
        veilidConnected: veilidConnected,
        contactsCount: status.contactsCount,
        emergenciesCount: status.emergenciesCount,
        safeHousesCount: status.safeHousesCount,
      );
    } catch (e) {
      debugPrint('❌ Status check failed: $e');
      rethrow;
    }
  }

  /// Add a contact
  /// identityString can be either:
  ///   - "forest aurora coffee" (fingerprint only, no messaging)
  ///   - "forest aurora coffee|VLD0:xyz..." (fingerprint + mailbox key for messaging)
  Future<void> addContact(String name, String identityString) async {
    try {
      // Parse identity string - it may contain fingerprint + mailbox key
      final parts = identityString.split('|');
      final fingerprint = parts[0].trim();
      final mailboxKey = parts.length > 1 ? parts[1].trim() : '';

      debugPrint('Adding contact: $name');
      debugPrint('  Fingerprint: $fingerprint');
      if (mailboxKey.isNotEmpty) {
        debugPrint('  Mailbox: ${mailboxKey.substring(0, 20)}...');
      } else {
        debugPrint('  Mailbox: (none - messaging disabled)');
      }

      // Call Rust FFI
      await _api.crateApiAddContact(
        name: name,
        fingerprintWords: fingerprint,
        mailboxKey: mailboxKey,
      );

      debugPrint('✅ Contact added: $name');
    } catch (e) {
      debugPrint('❌ Failed to add contact: $e');
      rethrow;
    }
  }

  /// Get all contacts
  Future<List<api.ContactInfo>> getContacts() async {
    try {
      // Call Rust FFI
      final contacts = await _api.crateApiGetContacts();

      debugPrint('📋 Retrieved ${contacts.length} contacts');
      return contacts;
    } catch (e) {
      debugPrint('❌ Failed to get contacts: $e');
      rethrow;
    }
  }

  /// Send an encrypted message to a contact
  Future<String> sendMessage(String contactId, String content) async {
    try {
      debugPrint('Sending message to: $contactId');

      // Call Rust FFI to create and encrypt the message
      final messageId = await _api.crateApiSendMessage(
        contactId: contactId,
        content: content,
      );

      debugPrint('✅ Message encrypted: $messageId');

      // Ensure we have a mailbox
      final hasMailbox = await _ensureMailbox();

      // Check if we can transmit via Veilid
      final veilidConnected = Platform.isAndroid || Platform.isIOS
          ? _veilidService.isConnected
          : true; // Desktop Veilid is managed by Rust and always connected

      debugPrint('🔍 Veilid transmission check:');
      debugPrint('   Platform: ${Platform.isAndroid ? "Android" : Platform.isIOS ? "iOS" : "Desktop"}');
      debugPrint('   Veilid connected: $veilidConnected');
      debugPrint('   Has mailbox: $hasMailbox');

      if (veilidConnected && hasMailbox) {
        try {
          // Get the recipient's mailbox key from database
          final recipientMailboxKey = await _api.crateApiGetContactMailboxKey(contactId: contactId);

          if (recipientMailboxKey != null && recipientMailboxKey.isNotEmpty) {
            // Get the encrypted message data from FFI
            final encryptedData = await _api.crateApiGetMessageForVeilid(
              contactId: contactId,
              messageId: messageId,
            );

            // Send via Veilid DHT (platform-specific)
            bool sent;
            if (Platform.isAndroid || Platform.isIOS) {
              // Mobile: Use veilid-flutter
              sent = await _messagingService.sendMessage(
                recipientMailboxKey,
                messageId,
                encryptedData,
              );
            } else {
              // Desktop: Use Rust FFI
              sent = await _api.crateApiSendMessageViaVeilidDesktop(
                recipientMailboxKey: recipientMailboxKey,
                messageData: encryptedData,
              );
            }

            if (sent) {
              debugPrint('📤 Message transmitted via Veilid');
            } else {
              debugPrint('⚠️ Message saved locally but Veilid transmission failed');
            }
          } else {
            debugPrint('⚠️ No mailbox key for recipient - message saved locally only');
          }
        } catch (e) {
          debugPrint('⚠️ Veilid transmission error: $e');
          debugPrint('   Message saved locally, will retry later');
        }
      } else {
        debugPrint('⚠️ Veilid transmission skipped - not connected or no mailbox');
      }

      return messageId;
    } catch (e) {
      debugPrint('❌ Failed to send message: $e');
      rethrow;
    }
  }

  /// Poll Veilid for new messages and save them to database (all platforms)
  Future<void> _pollVeilidMessages() async {
    try {
      // Check if we have a mailbox
      final mailboxKey = await _api.crateApiGetMailboxKey();
      if (mailboxKey == null || mailboxKey.isEmpty) {
        return; // No mailbox to poll
      }

      // Poll for new messages (platform-specific)
      List<Uint8List> encryptedMessages;

      if (Platform.isAndroid || Platform.isIOS) {
        // Mobile: Use veilid-flutter
        if (!_veilidService.isConnected || !_messagingService.hasMailbox) {
          return;
        }
        encryptedMessages = await _messagingService.pollMessages();
      } else {
        // Desktop: Use Rust FFI
        encryptedMessages = await _api.crateApiPollVeilidMailboxDesktop(mailboxKey: mailboxKey);
      }

      if (encryptedMessages.isEmpty) {
        return;
      }

      debugPrint('📥 Retrieved ${encryptedMessages.length} encrypted messages from Veilid');

      // For each message, decrypt and save using FFI
      for (final encryptedData in encryptedMessages) {
        try {
          final messageId = await _api.crateApiDecryptAndSaveMessage(
            encryptedData: encryptedData,
          );

          debugPrint('✅ Message decrypted and saved: $messageId');
        } catch (e) {
          debugPrint('⚠️ Failed to process message: $e');
        }
      }

    } catch (e) {
      debugPrint('⚠️ Failed to poll Veilid messages: $e');
    }
  }

  /// Get messages from a conversation
  Future<List<api.MessageInfo>> getMessages(String contactId, {int limit = 50}) async {
    try {
      // First, poll Veilid for any new messages
      await _pollVeilidMessages();

      // Call Rust FFI to get messages from database
      final messages = await _api.crateApiGetMessages(
        contactId: contactId,
        limit: limit,
      );

      debugPrint('📬 Retrieved ${messages.length} messages');
      return messages;
    } catch (e) {
      debugPrint('❌ Failed to get messages: $e');
      rethrow;
    }
  }

  /// Get all conversations
  Future<List<api.ConversationInfo>> getConversations() async {
    try {
      // First, poll Veilid for any new messages
      await _pollVeilidMessages();

      // Call Rust FFI
      final conversations = await _api.crateApiGetConversations();

      debugPrint('💬 Retrieved ${conversations.length} conversations');
      return conversations;
    } catch (e) {
      debugPrint('❌ Failed to get conversations: $e');
      rethrow;
    }
  }

  /// Mark a message as read
  Future<void> markMessageRead(String messageId) async {
    try {
      // Call Rust FFI
      await _api.crateApiMarkMessageRead(messageId: messageId);
    } catch (e) {
      debugPrint('❌ Failed to mark message as read: $e');
      rethrow;
    }
  }

  /// Shutdown (cleanup)
  Future<void> shutdown() async {
    try {
      debugPrint('Shutting down Underground Railroad...');

      // Shutdown Veilid (mobile only - desktop is handled by Rust FFI)
      if (Platform.isAndroid || Platform.isIOS) {
        await _veilidService.shutdown();
      }

      // Call Rust FFI (handles Veilid shutdown on desktop)
      await _api.crateApiShutdown();

      _initialized = false;
      _fingerprint = null;

      debugPrint('✅ Shutdown complete');
    } catch (e) {
      debugPrint('❌ Shutdown failed: $e');
    }
  }
}

/// Network status
class NetworkStatus {
  final bool veilidConnected;
  final int contactsCount;
  final int emergenciesCount;
  final int safeHousesCount;

  NetworkStatus({
    required this.veilidConnected,
    required this.contactsCount,
    required this.emergenciesCount,
    required this.safeHousesCount,
  });
}
