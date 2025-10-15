import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:underground_railroad/core/storage/database_service.dart';
import 'package:underground_railroad/core/storage/secure_storage_service.dart';

/// Duress mode manager
/// Handles switching between real and decoy databases
class DuressManager {
  final DatabaseService _databaseService;
  final SecureStorageService _secureStorage;

  bool _isDuressMode = false;

  DuressManager({
    required DatabaseService databaseService,
    required SecureStorageService secureStorage,
  })  : _databaseService = databaseService,
        _secureStorage = secureStorage;

  bool get isDuressMode => _isDuressMode;

  /// Activate duress mode
  /// This switches to the decoy database
  Future<void> activateDuressMode() async {
    if (_isDuressMode) {
      return;
    }

    await _databaseService.switchToDecoyMode();
    _isDuressMode = true;
  }

  /// Deactivate duress mode (return to real database)
  /// This should only be possible through secure re-authentication
  Future<void> deactivateDuressMode() async {
    if (!_isDuressMode) {
      return;
    }

    await _databaseService.switchToRealMode();
    _isDuressMode = false;
  }

  /// Generate decoy data for the decoy database
  /// This creates fake but plausible contacts and messages
  Future<void> generateDecoyData() async {
    if (!_isDuressMode) {
      await _databaseService.switchToDecoyMode();
    }

    final db = _databaseService.database;

    // Create decoy contacts
    final decoyContacts = [
      {
        'id': 'decoy_1',
        'name': 'Mom',
        'veilid_route': 'fake_route_1',
        'public_key': 'fake_pubkey_1',
        'safety_number': '123456',
        'verified': 1,
        'trust_level': 3,
        'created_at': DateTime.now().millisecondsSinceEpoch,
        'updated_at': DateTime.now().millisecondsSinceEpoch,
      },
      {
        'id': 'decoy_2',
        'name': 'Sarah',
        'veilid_route': 'fake_route_2',
        'public_key': 'fake_pubkey_2',
        'safety_number': '654321',
        'verified': 1,
        'trust_level': 2,
        'created_at': DateTime.now().millisecondsSinceEpoch,
        'updated_at': DateTime.now().millisecondsSinceEpoch,
      },
      {
        'id': 'decoy_3',
        'name': 'Work Group',
        'veilid_route': 'fake_route_3',
        'public_key': 'fake_pubkey_3',
        'safety_number': '789012',
        'verified': 0,
        'trust_level': 1,
        'created_at': DateTime.now().millisecondsSinceEpoch,
        'updated_at': DateTime.now().millisecondsSinceEpoch,
      },
    ];

    for (final contact in decoyContacts) {
      await db.insert('contacts', contact);
    }

    // Create decoy messages
    final now = DateTime.now();
    final decoyMessages = [
      {
        'id': 'msg_1',
        'contact_id': 'decoy_1',
        'content': 'How are you doing?',
        'sender_id': 'decoy_1',
        'recipient_id': 'self',
        'timestamp': now.subtract(const Duration(hours: 2)).millisecondsSinceEpoch,
        'is_sent': 1,
        'is_delivered': 1,
        'is_read': 1,
        'is_ephemeral': 0,
        'created_at': now.millisecondsSinceEpoch,
      },
      {
        'id': 'msg_2',
        'contact_id': 'decoy_1',
        'content': "I'm doing well, thanks!",
        'sender_id': 'self',
        'recipient_id': 'decoy_1',
        'timestamp': now.subtract(const Duration(hours: 1)).millisecondsSinceEpoch,
        'is_sent': 1,
        'is_delivered': 1,
        'is_read': 1,
        'is_ephemeral': 0,
        'created_at': now.millisecondsSinceEpoch,
      },
      {
        'id': 'msg_3',
        'contact_id': 'decoy_2',
        'content': 'Meeting at 3pm today?',
        'sender_id': 'decoy_2',
        'recipient_id': 'self',
        'timestamp': now.subtract(const Duration(minutes: 30)).millisecondsSinceEpoch,
        'is_sent': 1,
        'is_delivered': 1,
        'is_read': 0,
        'is_ephemeral': 0,
        'created_at': now.millisecondsSinceEpoch,
      },
    ];

    for (final message in decoyMessages) {
      await db.insert('messages', message);
    }

    if (!_isDuressMode) {
      await _databaseService.switchToRealMode();
    }
  }

  /// Check if decoy database has data
  Future<bool> hasDecoyData() async {
    final wasInDuressMode = _isDuressMode;

    if (!_isDuressMode) {
      await _databaseService.switchToDecoyMode();
    }

    final db = _databaseService.database;
    final result = await db.query('contacts', limit: 1);
    final hasData = result.isNotEmpty;

    if (!wasInDuressMode) {
      await _databaseService.switchToRealMode();
    }

    return hasData;
  }

  /// Emergency panic - wipe real data, keep decoy
  Future<void> panicWipe() async {
    // Wipe real database but keep decoy
    await _databaseService.emergencyWipe(wipeDecoy: false);

    // Wipe real encryption keys but keep decoy keys
    final decoyKey = await _secureStorage.getDatabaseKey(isDecoy: true);

    // Clear all secure storage
    await _secureStorage.deleteAll();

    // Restore only decoy key
    if (decoyKey != null) {
      await _secureStorage.storeDatabaseKey(decoyKey, isDecoy: false);
    }

    // Now in duress-only mode
    _isDuressMode = true;
  }
}

/// Provider for duress manager
final duressManagerProvider = Provider<DuressManager>((ref) {
  throw UnimplementedError('Must be overridden with actual instances');
});
