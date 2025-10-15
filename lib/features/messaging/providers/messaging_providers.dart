import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:underground_railroad/shared/models/message.dart';
import 'package:underground_railroad/shared/providers/app_providers.dart';

/// Send message action
final sendMessageProvider = Provider<Future<Message> Function({
  required String contactId,
  required String recipientRoute,
  required String content,
  required String senderId,
  required String recipientId,
  required String sharedSecret,
  MessageType messageType,
  bool isEphemeral,
  int? ephemeralDuration,
})>((ref) {
  final repository = ref.watch(messageRepositoryProvider);

  return ({
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
    return await repository.sendMessage(
      contactId: contactId,
      recipientRoute: recipientRoute,
      content: content,
      senderId: senderId,
      recipientId: recipientId,
      sharedSecret: sharedSecret,
      messageType: messageType,
      isEphemeral: isEphemeral,
      ephemeralDuration: ephemeralDuration,
    );
  };
});

/// Mark message as read action
final markMessageAsReadProvider = Provider<Future<void> Function(String)>((ref) {
  final repository = ref.watch(messageRepositoryProvider);
  return (messageId) => repository.markAsRead(messageId);
});

/// Mark all messages from contact as read
final markAllAsReadProvider = Provider<Future<void> Function(String)>((ref) {
  final repository = ref.watch(messageRepositoryProvider);
  return (contactId) => repository.markAllAsRead(contactId);
});

/// Delete message action
final deleteMessageProvider = Provider<Future<void> Function(String)>((ref) {
  final repository = ref.watch(messageRepositoryProvider);
  return (messageId) => repository.deleteMessage(messageId);
});

/// Cleanup ephemeral messages
final cleanupEphemeralMessagesProvider = Provider<Future<int> Function()>((ref) {
  final repository = ref.watch(messageRepositoryProvider);
  return () => repository.cleanupEphemeralMessages();
});
