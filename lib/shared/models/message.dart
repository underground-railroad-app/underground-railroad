import 'package:freezed_annotation/freezed_annotation.dart';

part 'message.freezed.dart';
part 'message.g.dart';

@freezed
class Message with _$Message {
  const factory Message({
    required String id,
    required String contactId,
    required String content,
    required String senderId,
    required String recipientId,
    required DateTime timestamp,
    @Default(false) bool isSent,
    @Default(false) bool isDelivered,
    @Default(false) bool isRead,
    @Default(false) bool isEphemeral,
    int? ephemeralDuration, // seconds until auto-delete
    MessageType? messageType,
    String? mediaUrl,
    required DateTime createdAt,
  }) = _Message;

  factory Message.fromJson(Map<String, dynamic> json) =>
      _$MessageFromJson(json);
}

enum MessageType {
  text,
  image,
  video,
  audio,
  file,
  location,
}

/// Encrypted message envelope for transmission
@freezed
class EncryptedMessage with _$EncryptedMessage {
  const factory EncryptedMessage({
    required String messageId,
    required String senderId,
    required String recipientId,
    required List<int> encryptedContent,
    required List<int> nonce,
    required String signature,
    required DateTime timestamp,
  }) = _EncryptedMessage;

  factory EncryptedMessage.fromJson(Map<String, dynamic> json) =>
      _$EncryptedMessageFromJson(json);
}

/// Message envelope for Veilid transmission
@freezed
class VeilidMessageEnvelope with _$VeilidMessageEnvelope {
  const factory VeilidMessageEnvelope({
    required String recipientRoute,
    required EncryptedMessage encryptedMessage,
    @Default(MessagePriority.normal) MessagePriority priority,
  }) = _VeilidMessageEnvelope;

  factory VeilidMessageEnvelope.fromJson(Map<String, dynamic> json) =>
      _$VeilidMessageEnvelopeFromJson(json);
}

enum MessagePriority {
  low,
  normal,
  high,
  urgent,
}
