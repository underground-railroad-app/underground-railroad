import 'package:flutter/foundation.dart';
import '../services/railroad_service.dart';
import '../ffi/api.dart' as api;

/// Global application state
class AppState extends ChangeNotifier {
  final RailroadService _service = RailroadService();

  bool _isInitialized = false;
  String? _userName;
  String? _fingerprint;
  bool _veilidConnected = false;
  List<api.ContactInfo> _contacts = [];
  int _emergencyCount = 0;
  int _safeHouseCount = 0;

  bool get isInitialized => _isInitialized;
  String? get userName => _userName;
  String? get fingerprint => _fingerprint;
  bool get veilidConnected => _veilidConnected;
  List<api.ContactInfo> get contacts => _contacts;
  int get contactCount => _contacts.length;
  int get emergencyCount => _emergencyCount;
  int get safeHouseCount => _safeHouseCount;

  /// Initialize the app (create identity + start Veilid)
  Future<void> initialize(String name, String password) async {
    try {
      // Initialize via Rust FFI (creates identity, starts Veilid)
      // RailroadService handles data directory internally
      await _service.initialize(name, password);
      final fingerprint = _service.fingerprint;

      _isInitialized = true;
      _userName = name;
      _fingerprint = fingerprint;

      debugPrint('✅ Initialized! Veilid starting in background...');

      // Load existing data from database
      await _loadDataFromDatabase();

      // Start polling for status updates
      _startStatusPolling();

      notifyListeners();
    } catch (e) {
      debugPrint('❌ Initialization failed: $e');
      rethrow;
    }
  }

  /// Load data from database after login
  Future<void> _loadDataFromDatabase() async {
    try {
      // Load contacts
      _contacts = await _service.getContacts();

      debugPrint('📦 Loaded ${_contacts.length} contacts from database');
      for (final contact in _contacts) {
        debugPrint('   - ${contact.name}: ${contact.fingerprint}');
      }

      // TODO: Load emergencies and safe houses when those methods are added

      notifyListeners();
    } catch (e) {
      debugPrint('⚠️ Failed to load data from database: $e');
    }
  }

  /// Refresh contacts from database
  Future<void> refreshContacts() async {
    try {
      _contacts = await _service.getContacts();
      debugPrint('🔄 Refreshed contacts: ${_contacts.length}');
      notifyListeners();
    } catch (e) {
      debugPrint('⚠️ Failed to refresh contacts: $e');
    }
  }

  /// Load existing identity
  Future<bool> loadIdentity(String password) async {
    // TODO: Call Rust FFI to load identity
    return false;
  }

  /// Poll network status every few seconds
  void _startStatusPolling() {
    Future.delayed(const Duration(seconds: 3), () async {
      if (!_isInitialized) return;

      try {
        final status = await _service.getStatus();
        _veilidConnected = status.veilidConnected;

        // Update counts from database
        final actualContactCount = status.contactsCount;
        if (actualContactCount != _contacts.length) {
          // Contact count changed - refresh the list
          await refreshContacts();
        }

        _emergencyCount = status.emergenciesCount;
        _safeHouseCount = status.safeHousesCount;

        debugPrint('📊 Status update: Veilid=${_veilidConnected ? "🟢" : "🔴"}, Contacts=${_contacts.length}');

        notifyListeners();

        // Continue polling
        _startStatusPolling();
      } catch (e) {
        debugPrint('Status poll failed: $e');
        // Retry anyway
        _startStatusPolling();
      }
    });
  }

  /// Update network stats
  void updateStats({
    int? emergencies,
    int? safeHouses,
  }) {
    // Contacts are now managed by refreshing from database
    // Use refreshContacts() instead
    if (emergencies != null) _emergencyCount = emergencies;
    if (safeHouses != null) _safeHouseCount = safeHouses;
    notifyListeners();
  }

  Future<String> _getDataDir() async {
    if (kIsWeb) {
      return '/tmp/underground-railroad'; // Web storage
    } else {
      // TODO: Use path_provider
      return '~/.underground-railroad';
    }
  }

  @override
  void dispose() {
    _service.shutdown();
    super.dispose();
  }
}

