import 'dart:convert';
import 'package:crypto/crypto.dart';
import 'package:underground_railroad/core/constants/app_constants.dart';
import 'package:underground_railroad/core/crypto/crypto_service.dart';
import 'package:underground_railroad/core/storage/secure_storage_service.dart';

/// Security manager for authentication and encryption key management
class SecurityManager {
  final SecureStorageService _secureStorage;
  final CryptoService _crypto;

  SecurityManager({
    required SecureStorageService secureStorage,
    required CryptoService crypto,
  })  : _secureStorage = secureStorage,
        _crypto = crypto;

  /// Check if system is initialized (has PIN set)
  Future<bool> isInitialized() async {
    return await _secureStorage.isInitialized();
  }

  /// Initialize system with PIN and optional duress PIN
  Future<void> initializeWithPIN({
    required String pin,
    String? duressPin,
  }) async {
    if (pin.length < AppConstants.pinMinLength) {
      throw ArgumentError('PIN too short');
    }

    if (duressPin != null && duressPin.length < AppConstants.pinMinLength) {
      throw ArgumentError('Duress PIN too short');
    }

    // Generate master salt
    final salt = await _crypto.generateSalt();
    final saltBase64 = base64Encode(salt);
    await _secureStorage.storeMasterSalt(saltBase64);

    // Hash PIN with salt
    final pinHash = _hashPIN(pin, salt);
    await _secureStorage.storePinHash(pinHash);

    // Generate database encryption key from PIN
    final dbKey = await _crypto.deriveKey(pin, salt);
    final dbKeyBase64 = base64Encode(dbKey);
    await _secureStorage.storeDatabaseKey(dbKeyBase64, isDecoy: false);

    // If duress PIN provided, set it up
    if (duressPin != null) {
      final duressPinHash = _hashPIN(duressPin, salt);
      await _secureStorage.storeDuressPinHash(duressPinHash);

      // Generate separate decoy database key
      final decoyDbKey = await _crypto.deriveKey(duressPin, salt);
      final decoyDbKeyBase64 = base64Encode(decoyDbKey);
      await _secureStorage.storeDatabaseKey(decoyDbKeyBase64, isDecoy: true);
    }
  }

  /// Verify PIN and return whether it's real or duress
  Future<PINVerificationResult> verifyPIN(String pin) async {
    final saltBase64 = await _secureStorage.getMasterSalt();
    if (saltBase64 == null) {
      throw StateError('System not initialized');
    }

    final salt = base64Decode(saltBase64);
    final pinHash = _hashPIN(pin, salt);

    // Check if it matches real PIN
    final realPinHash = await _secureStorage.getPinHash();
    if (realPinHash != null && pinHash == realPinHash) {
      return PINVerificationResult.real;
    }

    // Check if it matches duress PIN
    final duressPinHash = await _secureStorage.getDuressPinHash();
    if (duressPinHash != null && pinHash == duressPinHash) {
      return PINVerificationResult.duress;
    }

    return PINVerificationResult.invalid;
  }

  /// Get database encryption key
  Future<String> getDatabaseKey({bool isDecoy = false}) async {
    final key = await _secureStorage.getDatabaseKey(isDecoy: isDecoy);
    if (key == null) {
      throw StateError('Database key not found');
    }
    return key;
  }

  /// Change PIN (maintains same encryption keys)
  Future<void> changePIN({
    required String oldPin,
    required String newPin,
  }) async {
    // Verify old PIN first
    final verificationResult = await verifyPIN(oldPin);
    if (verificationResult == PINVerificationResult.invalid) {
      throw Exception('Invalid PIN');
    }

    if (verificationResult == PINVerificationResult.duress) {
      throw Exception('Cannot change PIN in duress mode');
    }

    // Get salt
    final saltBase64 = await _secureStorage.getMasterSalt();
    if (saltBase64 == null) {
      throw StateError('System not initialized');
    }

    final salt = base64Decode(saltBase64);

    // Hash new PIN
    final newPinHash = _hashPIN(newPin, salt);
    await _secureStorage.storePinHash(newPinHash);

    // Re-derive database key with new PIN
    final newDbKey = await _crypto.deriveKey(newPin, salt);
    final newDbKeyBase64 = base64Encode(newDbKey);
    await _secureStorage.storeDatabaseKey(newDbKeyBase64, isDecoy: false);
  }

  /// Emergency panic - wipe all sensitive data
  Future<void> panic() async {
    await _secureStorage.emergencyWipe();
  }

  /// Hash PIN using SHA-256 (for comparison, not encryption)
  String _hashPIN(String pin, List<int> salt) {
    final combined = utf8.encode(pin) + salt;
    final digest = sha256.convert(combined);
    return digest.toString();
  }
}

/// PIN verification result
enum PINVerificationResult {
  real,
  duress,
  invalid,
}
