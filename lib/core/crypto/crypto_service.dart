import 'dart:typed_data';
import 'package:underground_railroad/generated/bridge.dart/api.dart';

/// Cryptographic service that wraps Rust crypto functions
class CryptoService {
  // Singleton pattern
  static final CryptoService _instance = CryptoService._internal();
  factory CryptoService() => _instance;
  CryptoService._internal();

  /// Derive encryption key from password and salt
  Future<Uint8List> deriveKey(String password, Uint8List salt) async {
    return await deriveEncryptionKey(password: password, salt: salt);
  }

  /// Generate random salt for key derivation
  Future<Uint8List> generateSalt() async {
    return await generateKeySalt();
  }

  /// Generate secure random bytes
  Future<Uint8List> generateRandomBytes(int length) async {
    return await generateSecureRandom(length: BigInt.from(length));
  }

  /// Encrypt data using ChaCha20-Poly1305
  Future<Uint8List> encrypt(Uint8List key, Uint8List plaintext) async {
    if (key.length != 32) {
      throw ArgumentError('Key must be 32 bytes');
    }
    return await encryptBytes(key: key, plaintext: plaintext);
  }

  /// Decrypt data using ChaCha20-Poly1305
  Future<Uint8List> decrypt(Uint8List key, Uint8List ciphertext) async {
    if (key.length != 32) {
      throw ArgumentError('Key must be 32 bytes');
    }
    return await decryptBytes(key: key, ciphertext: ciphertext);
  }

  /// Hash data using Blake3
  Future<Uint8List> hash(Uint8List data) async {
    return await hashData(data: data);
  }

  /// Check system health
  Future<bool> cryptoHealthCheck() async {
    final status = await healthCheck();
    return status.contains('OK');
  }
}
