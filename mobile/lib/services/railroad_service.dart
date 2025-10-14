import 'package:flutter/foundation.dart';
import 'dart:io' show Directory, Platform;
import 'package:path_provider/path_provider.dart';

// Native platforms only - web intentionally excluded for security
import '../ffi/frb_generated.dart';
import '../ffi/api.dart' as api;
import 'veilid_service.dart';

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

  bool _initialized = false;
  String? _fingerprint;

  bool get isInitialized => _initialized;
  String? get fingerprint => _fingerprint;

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
        dataDir: baseDataDir,
      );

      _fingerprint = fingerprint;
      _initialized = true;

      debugPrint('✅ Initialized! Fingerprint: $_fingerprint');

      // Initialize Veilid with correct two-step pattern (fixes Android crash)
      await _veilidService.initialize('UndergroundRailroad');

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
      // Call Rust FFI
      final status = await _api.crateApiGetStatus();

      return NetworkStatus(
        veilidConnected: status.veilidConnected,
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
  Future<void> addContact(String name, String fingerprint) async {
    try {
      debugPrint('Adding contact: $name with fingerprint: $fingerprint');

      // Call Rust FFI
      await _api.crateApiAddContact(
        name: name,
        fingerprintWords: fingerprint,
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

      // Call Rust FFI
      final messageId = await _api.crateApiSendMessage(
        contactId: contactId,
        content: content,
      );

      debugPrint('✅ Message sent: $messageId');
      return messageId;
    } catch (e) {
      debugPrint('❌ Failed to send message: $e');
      rethrow;
    }
  }

  /// Get messages from a conversation
  Future<List<api.MessageInfo>> getMessages(String contactId, {int limit = 50}) async {
    try {
      // Call Rust FFI
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

      // Shutdown Veilid
      await _veilidService.shutdown();

      // Call Rust FFI
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
