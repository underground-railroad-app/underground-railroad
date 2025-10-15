import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:underground_railroad/core/constants/app_constants.dart';

/// Secure storage service for sensitive data (keys, credentials, etc.)
/// Uses platform-specific secure storage (Keychain on iOS, Keystore on Android)
class SecureStorageService {
  // Singleton pattern
  static final SecureStorageService _instance = SecureStorageService._internal();
  factory SecureStorageService() => _instance;
  SecureStorageService._internal();

  late final FlutterSecureStorage _storage;

  /// Initialize secure storage
  Future<void> initialize() async {
    _storage = const FlutterSecureStorage(
      aOptions: AndroidOptions(
        encryptedSharedPreferences: true,
        resetOnError: true,
      ),
      iOptions: IOSOptions(
        accessibility: KeychainAccessibility.first_unlock_this_device,
        synchronizable: false,
      ),
      mOptions: MacOsOptions(
        accessibility: KeychainAccessibility.first_unlock_this_device,
        synchronizable: false,
      ),
      lOptions: LinuxOptions(
        resetOnError: true,
      ),
    );
  }

  /// Store a value
  Future<void> write(String key, String value) async {
    await _storage.write(key: key, value: value);
  }

  /// Read a value
  Future<String?> read(String key) async {
    return await _storage.read(key: key);
  }

  /// Delete a value
  Future<void> delete(String key) async {
    await _storage.delete(key: key);
  }

  /// Delete all values (for panic mode)
  Future<void> deleteAll() async {
    await _storage.deleteAll();
  }

  /// Check if a key exists
  Future<bool> containsKey(String key) async {
    final value = await read(key);
    return value != null;
  }

  // Convenience methods for specific keys

  Future<void> storeMasterSalt(String salt) async {
    await write(AppConstants.keyMasterSalt, salt);
  }

  Future<String?> getMasterSalt() async {
    return await read(AppConstants.keyMasterSalt);
  }

  Future<void> storePinHash(String hash) async {
    await write(AppConstants.keyPinHash, hash);
  }

  Future<String?> getPinHash() async {
    return await read(AppConstants.keyPinHash);
  }

  Future<void> storeDuressPinHash(String hash) async {
    await write(AppConstants.keyDuressPinHash, hash);
  }

  Future<String?> getDuressPinHash() async {
    return await read(AppConstants.keyDuressPinHash);
  }

  Future<void> storeDatabaseKey(String key, {bool isDecoy = false}) async {
    final storageKey = isDecoy
        ? AppConstants.keyDecoyDatabaseKey
        : AppConstants.keyDatabaseKey;
    await write(storageKey, key);
  }

  Future<String?> getDatabaseKey({bool isDecoy = false}) async {
    final storageKey = isDecoy
        ? AppConstants.keyDecoyDatabaseKey
        : AppConstants.keyDatabaseKey;
    return await read(storageKey);
  }

  Future<void> storeVeilidIdentity(String identity) async {
    await write(AppConstants.keyVeilidIdentity, identity);
  }

  Future<String?> getVeilidIdentity() async {
    return await read(AppConstants.keyVeilidIdentity);
  }

  /// Check if the system is initialized (has master salt)
  Future<bool> isInitialized() async {
    return await containsKey(AppConstants.keyMasterSalt);
  }

  /// Emergency wipe - deletes all secure storage
  /// Used for panic button or duress mode
  Future<void> emergencyWipe() async {
    await deleteAll();
  }
}
