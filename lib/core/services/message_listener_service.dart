import 'dart:async';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:underground_railroad/core/crypto/message_crypto_service.dart';
import 'package:underground_railroad/core/veilid/veilid_service.dart' hide veilidServiceProvider;
import 'package:underground_railroad/features/contacts/data/contact_repository.dart';
import 'package:underground_railroad/features/messaging/data/message_repository.dart';
import 'package:underground_railroad/shared/models/contact.dart';
import 'package:underground_railroad/shared/models/message.dart';
import 'package:underground_railroad/shared/providers/app_providers.dart';

/// Service for listening to incoming messages via Veilid
class MessageListenerService {
  final VeilidService _veilid;
  final MessageRepository _messageRepo;
  final ContactRepository _contactRepo;
  final MessageCryptoService _crypto;

  StreamController<Message>? _messageController;
  Timer? _pollingTimer;
  bool _isListening = false;

  MessageListenerService({
    required VeilidService veilid,
    required MessageRepository messageRepo,
    required ContactRepository contactRepo,
    required MessageCryptoService crypto,
  })  : _veilid = veilid,
        _messageRepo = messageRepo,
        _contactRepo = contactRepo,
        _crypto = crypto;

  /// Start listening for incoming messages
  Future<void> startListening() async {
    if (_isListening) return;

    _isListening = true;
    _messageController = StreamController<Message>.broadcast();

    // TODO: Real implementation would subscribe to Veilid updates
    // For now, use polling as a fallback
    _startPolling();
  }

  /// Stop listening for messages
  Future<void> stopListening() async {
    _isListening = false;
    _pollingTimer?.cancel();
    await _messageController?.close();
    _messageController = null;
  }

  /// Get stream of incoming messages
  Stream<Message>? get messageStream => _messageController?.stream;

  /// Poll for new messages (fallback mechanism)
  void _startPolling() {
    _pollingTimer = Timer.periodic(const Duration(seconds: 5), (_) {
      _checkForMessages();
    });
  }

  /// Check for new messages via Veilid
  Future<void> _checkForMessages() async {
    if (!_veilid.isConnected) return;

    try {
      // TODO: Real implementation:
      // 1. Check our private route for incoming messages
      // 2. Retrieve encrypted message data
      // 3. Decrypt and process each message

      // Placeholder: In real implementation, this would be:
      // final incomingData = await _veilid.checkPrivateRoute();
      // for (final data in incomingData) {
      //   await _processIncomingMessage(data);
      // }
    } catch (e) {
      print('Error checking for messages: $e');
    }
  }

  /// Process an incoming encrypted message
  Future<void> _processIncomingMessage(List<int> encryptedData) async {
    try {
      // Deserialize encrypted message
      final encryptedMessage = _crypto.deserializeEncryptedMessage(encryptedData);

      // Find contact by sender ID
      final contact = await _findContactBySenderId(encryptedMessage.senderId);
      if (contact == null) {
        print('Unknown sender: ${encryptedMessage.senderId}');
        return;
      }

      // Derive shared secret
      // TODO: Get our private key from secure storage
      final myPrivateKey = 'TODO'; // Get from identity
      final sharedSecret = await _crypto.deriveSharedSecret(
        myPrivateKey: myPrivateKey,
        theirPublicKey: contact.publicKey,
      );

      // Decrypt and store message
      final message = await _messageRepo.receiveMessage(
        contactId: contact.id,
        encryptedData: encryptedData,
        sharedSecret: sharedSecret,
      );

      // Emit message to stream
      _messageController?.add(message);

      // TODO: Show notification
      await _showNotification(contact, message);
    } catch (e) {
      print('Error processing incoming message: $e');
    }
  }

  /// Find contact by sender ID (public key)
  Future<Contact?> _findContactBySenderId(String senderId) async {
    final contacts = await _contactRepo.getContacts();
    return contacts.cast<Contact?>().firstWhere(
          (c) => c?.publicKey == senderId,
          orElse: () => null,
        );
  }

  /// Show notification for new message
  Future<void> _showNotification(Contact contact, Message message) async {
    // TODO: Integrate with notification service
    print('New message from ${contact.name}: ${message.content}');
  }

  /// Manually check for messages (pull)
  Future<void> checkNow() async {
    await _checkForMessages();
  }

  void dispose() {
    stopListening();
  }
}

/// Provider for message listener service
final messageListenerServiceProvider = Provider<MessageListenerService>((ref) {
  final veilid = ref.watch(veilidServiceProvider);
  final messageRepo = ref.watch(messageRepositoryProvider);
  final contactRepo = ref.watch(contactRepositoryProvider);
  final crypto = ref.watch(messageCryptoServiceProvider);

  final service = MessageListenerService(
    veilid: veilid,
    messageRepo: messageRepo,
    contactRepo: contactRepo,
    crypto: crypto,
  );

  // Auto-cleanup
  ref.onDispose(() {
    service.dispose();
  });

  return service;
});
