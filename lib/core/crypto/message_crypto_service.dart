import 'dart:convert';
import 'dart:typed_data';
import 'package:crypto/crypto.dart';
import 'package:underground_railroad/core/crypto/crypto_service.dart';
import 'package:underground_railroad/shared/models/message.dart';

/// End-to-end encryption service for messages
/// Uses ChaCha20-Poly1305 with per-contact shared secrets
class MessageCryptoService {
  final CryptoService _crypto;

  MessageCryptoService({required CryptoService crypto}) : _crypto = crypto;

  /// Encrypt a message for a specific contact
  /// Returns an EncryptedMessage ready for transmission
  Future<EncryptedMessage> encryptMessage({
    required Message message,
    required String sharedSecret,
    required String senderId,
  }) async {
    // Convert message to JSON
    final messageJson = message.toJson();
    final messageString = jsonEncode(messageJson);
    final plaintext = utf8.encode(messageString);

    // Derive encryption key from shared secret
    final salt = await _crypto.generateSalt();
    final encryptionKey = await _crypto.deriveKey(sharedSecret, salt);

    // Encrypt the message content
    final encryptedContent = await _crypto.encrypt(
      encryptionKey,
      Uint8List.fromList(plaintext),
    );

    // Generate signature for authentication
    final signature = await _generateSignature(
      encryptedContent,
      encryptionKey,
    );

    return EncryptedMessage(
      messageId: message.id,
      senderId: senderId,
      recipientId: message.recipientId,
      encryptedContent: encryptedContent.toList(),
      nonce: salt.toList(),
      signature: signature,
      timestamp: DateTime.now(),
    );
  }

  /// Decrypt a received encrypted message
  /// Verifies signature and returns the original Message
  Future<Message> decryptMessage({
    required EncryptedMessage encryptedMessage,
    required String sharedSecret,
  }) async {
    // Derive decryption key from shared secret and nonce
    final salt = Uint8List.fromList(encryptedMessage.nonce);
    final decryptionKey = await _crypto.deriveKey(sharedSecret, salt);

    // Verify signature
    final expectedSignature = await _generateSignature(
      Uint8List.fromList(encryptedMessage.encryptedContent),
      decryptionKey,
    );

    if (expectedSignature != encryptedMessage.signature) {
      throw Exception('Message signature verification failed');
    }

    // Decrypt the message content
    final decrypted = await _crypto.decrypt(
      decryptionKey,
      Uint8List.fromList(encryptedMessage.encryptedContent),
    );

    // Parse JSON back to Message
    final messageString = utf8.decode(decrypted);
    final messageJson = jsonDecode(messageString) as Map<String, dynamic>;

    return Message.fromJson(messageJson);
  }

  /// Generate HMAC signature for message authentication
  Future<String> _generateSignature(
    Uint8List data,
    Uint8List key,
  ) async {
    // Use Blake3 hash of data + key for signature
    final combined = Uint8List.fromList([...data, ...key]);
    final hash = await _crypto.hash(combined);
    return base64Encode(hash);
  }

  /// Derive shared secret from two parties' keys (simplified DH)
  /// In production, use proper ECDH key exchange
  Future<String> deriveSharedSecret({
    required String myPrivateKey,
    required String theirPublicKey,
  }) async {
    // TODO: Implement proper ECDH
    // For now, hash both keys together as placeholder
    final combined = '$myPrivateKey:$theirPublicKey';
    final hash = await _crypto.hash(utf8.encode(combined) as Uint8List);
    return base64Encode(hash);
  }

  /// Generate safety number for contact verification
  /// Used for out-of-band verification
  Future<String> generateSafetyNumber({
    required String myPublicKey,
    required String theirPublicKey,
  }) async {
    // Combine and hash public keys
    final combined = '$myPublicKey:$theirPublicKey';
    final hash = await _crypto.hash(utf8.encode(combined) as Uint8List);

    // Convert to 6-digit safety number (like Signal)
    final bytes = hash.sublist(0, 4);
    final number = bytes.buffer.asByteData().getUint32(0, Endian.big);
    return (number % 1000000).toString().padLeft(6, '0');
  }

  /// Serialize EncryptedMessage for Veilid transmission
  List<int> serializeEncryptedMessage(EncryptedMessage message) {
    final json = message.toJson();
    final jsonString = jsonEncode(json);
    return utf8.encode(jsonString);
  }

  /// Deserialize EncryptedMessage from Veilid data
  EncryptedMessage deserializeEncryptedMessage(List<int> data) {
    final jsonString = utf8.decode(data);
    final json = jsonDecode(jsonString) as Map<String, dynamic>;
    return EncryptedMessage.fromJson(json);
  }
}
