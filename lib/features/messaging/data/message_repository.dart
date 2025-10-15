import 'dart:convert';
import 'package:sqflite_sqlcipher/sqflite.dart';
import 'package:underground_railroad/core/crypto/message_crypto_service.dart';
import 'package:underground_railroad/core/storage/database_service.dart';
import 'package:underground_railroad/core/veilid/veilid_service.dart';
import 'package:underground_railroad/shared/models/message.dart';
import 'package:uuid/uuid.dart';

class MessageRepository {
  final DatabaseService _db;
  final VeilidService _veilid;
  final MessageCryptoService _crypto;

  MessageRepository({
    required DatabaseService db,
    required VeilidService veilid,
    required MessageCryptoService crypto,
  })  : _db = db,
        _veilid = veilid,
        _crypto = crypto;

  /// Get messages for a contact
  Future<List<Message>> getMessagesForContact(String contactId) async {
    final db = _db.database;
    final results = await db.query(
      'messages',
      where: 'contact_id = ?',
      whereArgs: [contactId],
      orderBy: 'timestamp DESC',
    );

    return results.map((map) => _messageFromMap(map)).toList();
  }

  /// Get recent messages across all contacts
  Future<List<Message>> getRecentMessages({int limit = 50}) async {
    final db = _db.database;
    final results = await db.query(
      'messages',
      orderBy: 'timestamp DESC',
      limit: limit,
    );

    return results.map((map) => _messageFromMap(map)).toList();
  }

  /// Send encrypted message via Veilid
  Future<Message> sendMessage({
    required String contactId,
    required String recipientRoute,
    required String content,
    required String senderId,
    required String recipientId,
    required String sharedSecret,
    MessageType messageType = MessageType.text,
    bool isEphemeral = false,
    int? ephemeralDuration,
  }) async {
    final now = DateTime.now();

    // Create message
    final message = Message(
      id: const Uuid().v4(),
      contactId: contactId,
      content: content,
      senderId: senderId,
      recipientId: recipientId,
      timestamp: now,
      messageType: messageType,
      isEphemeral: isEphemeral,
      ephemeralDuration: ephemeralDuration,
      createdAt: now,
    );

    // Encrypt message
    final encryptedMessage = await _crypto.encryptMessage(
      message: message,
      sharedSecret: sharedSecret,
      senderId: senderId,
    );

    // Send via Veilid
    final messageBytes = _crypto.serializeEncryptedMessage(encryptedMessage);
    await _veilid.sendMessage(recipientRoute, messageBytes);

    // Store in local database
    final db = _db.database;
    await db.insert('messages', _messageToMap(message.copyWith(isSent: true)));

    return message.copyWith(isSent: true);
  }

  /// Receive and decrypt message
  Future<Message> receiveMessage({
    required String contactId,
    required List<int> encryptedData,
    required String sharedSecret,
  }) async {
    // Deserialize encrypted message
    final encryptedMessage = _crypto.deserializeEncryptedMessage(encryptedData);

    // Decrypt message
    final message = await _crypto.decryptMessage(
      encryptedMessage: encryptedMessage,
      sharedSecret: sharedSecret,
    );

    // Store in local database
    final db = _db.database;
    await db.insert(
      'messages',
      _messageToMap(message.copyWith(
        contactId: contactId,
        isDelivered: true,
      )),
    );

    return message;
  }

  /// Mark message as read
  Future<void> markAsRead(String messageId) async {
    final db = _db.database;
    await db.update(
      'messages',
      {'is_read': 1},
      where: 'id = ?',
      whereArgs: [messageId],
    );
  }

  /// Mark all messages from contact as read
  Future<void> markAllAsRead(String contactId) async {
    final db = _db.database;
    await db.update(
      'messages',
      {'is_read': 1},
      where: 'contact_id = ? AND is_read = 0',
      whereArgs: [contactId],
    );
  }

  /// Delete message
  Future<void> deleteMessage(String messageId) async {
    final db = _db.database;
    await db.delete(
      'messages',
      where: 'id = ?',
      whereArgs: [messageId],
    );
  }

  /// Delete all messages with contact
  Future<void> deleteAllMessages(String contactId) async {
    final db = _db.database;
    await db.delete(
      'messages',
      where: 'contact_id = ?',
      whereArgs: [contactId],
    );
  }

  /// Clean up expired ephemeral messages
  Future<int> cleanupEphemeralMessages() async {
    final now = DateTime.now().millisecondsSinceEpoch;
    final db = _db.database;

    // Find expired ephemeral messages
    final results = await db.query(
      'messages',
      where: 'is_ephemeral = 1 AND ephemeral_duration IS NOT NULL',
    );

    int deleted = 0;
    for (final map in results) {
      final createdAt = map['created_at'] as int;
      final duration = map['ephemeral_duration'] as int;
      final expiresAt = createdAt + (duration * 1000); // Convert to ms

      if (now >= expiresAt) {
        await db.delete(
          'messages',
          where: 'id = ?',
          whereArgs: [map['id']],
        );
        deleted++;
      }
    }

    return deleted;
  }

  /// Get unread message count
  Future<int> getUnreadCount({String? contactId}) async {
    final db = _db.database;

    if (contactId != null) {
      final result = await db.rawQuery(
        'SELECT COUNT(*) as count FROM messages WHERE contact_id = ? AND is_read = 0',
        [contactId],
      );
      return Sqflite.firstIntValue(result) ?? 0;
    } else {
      final result = await db.rawQuery(
        'SELECT COUNT(*) as count FROM messages WHERE is_read = 0',
      );
      return Sqflite.firstIntValue(result) ?? 0;
    }
  }

  Message _messageFromMap(Map<String, dynamic> map) {
    return Message(
      id: map['id'] as String,
      contactId: map['contact_id'] as String,
      content: map['content'] as String,
      senderId: map['sender_id'] as String,
      recipientId: map['recipient_id'] as String,
      timestamp: DateTime.fromMillisecondsSinceEpoch(map['timestamp'] as int),
      isSent: (map['is_sent'] as int) == 1,
      isDelivered: (map['is_delivered'] as int) == 1,
      isRead: (map['is_read'] as int) == 1,
      isEphemeral: (map['is_ephemeral'] as int) == 1,
      ephemeralDuration: map['ephemeral_duration'] as int?,
      messageType: map['message_type'] != null
          ? MessageType.values.firstWhere(
              (e) => e.name == map['message_type'],
              orElse: () => MessageType.text,
            )
          : null,
      createdAt: DateTime.fromMillisecondsSinceEpoch(map['created_at'] as int),
    );
  }

  Map<String, dynamic> _messageToMap(Message message) {
    return {
      'id': message.id,
      'contact_id': message.contactId,
      'content': message.content,
      'sender_id': message.senderId,
      'recipient_id': message.recipientId,
      'timestamp': message.timestamp.millisecondsSinceEpoch,
      'is_sent': message.isSent ? 1 : 0,
      'is_delivered': message.isDelivered ? 1 : 0,
      'is_read': message.isRead ? 1 : 0,
      'is_ephemeral': message.isEphemeral ? 1 : 0,
      'ephemeral_duration': message.ephemeralDuration,
      'message_type': message.messageType?.name,
      'created_at': message.createdAt.millisecondsSinceEpoch,
    };
  }
}
