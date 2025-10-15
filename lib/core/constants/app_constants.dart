/// Application-wide constants
class AppConstants {
  // App metadata
  static const String appName = 'Underground Railroad';
  static const String appVersion = '0.1.0';

  // Cryptography
  static const int keySize = 32; // 256 bits
  static const int saltSize = 32; // 256 bits
  static const int nonceSize = 12; // 96 bits for ChaCha20-Poly1305

  // Security
  static const int pinMinLength = 6;
  static const int pinMaxLength = 12;
  static const int autoLockMinutes = 5;
  static const int maxFailedAttempts = 3;

  // Database
  static const String dbName = 'underground_railroad.db';
  static const String decoyDbName = 'underground_railroad_decoy.db';
  static const int dbVersion = 1;

  // Storage keys
  static const String keyMasterSalt = 'master_salt';
  static const String keyPinHash = 'pin_hash';
  static const String keyDuressPinHash = 'duress_pin_hash';
  static const String keyDatabaseKey = 'database_key';
  static const String keyDecoyDatabaseKey = 'decoy_database_key';
  static const String keyVeilidIdentity = 'veilid_identity';

  // Veilid
  static const String veilidConfigDir = 'veilid_config';

  // Limits
  static const int maxMessageLength = 65536; // 64KB
  static const int maxContactNameLength = 64;
  static const int maxGroupNameLength = 64;

  // Timeouts
  static const int networkTimeoutSeconds = 30;
  static const int messageRetryAttempts = 3;
}
