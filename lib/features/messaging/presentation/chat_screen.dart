import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:underground_railroad/core/services/message_listener_service.dart';
import 'package:underground_railroad/features/contacts/providers/contact_providers.dart';
import 'package:underground_railroad/shared/models/message.dart';
import 'package:underground_railroad/shared/providers/app_providers.dart';
import 'package:underground_railroad/features/messaging/providers/messaging_providers.dart';
import 'package:underground_railroad/features/messaging/providers/message_refresh_provider.dart';

class ChatScreen extends ConsumerStatefulWidget {
  final String contactId;

  const ChatScreen({
    super.key,
    required this.contactId,
  });

  @override
  ConsumerState<ChatScreen> createState() => _ChatScreenState();
}

class _ChatScreenState extends ConsumerState<ChatScreen> {
  final _messageController = TextEditingController();
  final _scrollController = ScrollController();

  @override
  void initState() {
    super.initState();
    // Listen for real-time message updates
    _setupMessageListener();
  }

  void _setupMessageListener() {
    final messageListener = ref.read(messageListenerServiceProvider);
    messageListener.messageStream?.listen((message) {
      // Check if message is for this contact
      if (message.contactId == widget.contactId) {
        // Mark as read
        final markAsRead = ref.read(markMessageAsReadProvider);
        markAsRead(message.id);

        // Refresh messages
        ref.invalidate(messagesProvider(widget.contactId));
      }
    });
  }

  @override
  void dispose() {
    _messageController.dispose();
    _scrollController.dispose();
    super.dispose();
  }

  void _handleSendMessage() async {
    final text = _messageController.text.trim();
    if (text.isEmpty) return;

    // Get contact
    final contactAsync = ref.read(contactProvider(widget.contactId));
    final contact = contactAsync.value;
    if (contact == null) return;

    // Get current identity
    final myIdentity = ref.read(currentIdentityProvider);
    if (myIdentity == null) {
      _showError('Identity not initialized');
      return;
    }

    // Clear input immediately for better UX
    final messageText = text;
    _messageController.clear();

    try {
      // Derive shared secret from keys
      final messageCrypto = ref.read(messageCryptoServiceProvider);
      final sharedSecret = await messageCrypto.deriveSharedSecret(
        myPrivateKey: myIdentity.secretKey,
        theirPublicKey: contact.publicKey,
      );

      // Send message
      final sendMessage = ref.read(sendMessageProvider);
      await sendMessage(
        contactId: contact.id,
        recipientRoute: contact.veilidRoute,
        content: messageText,
        senderId: myIdentity.publicKey,
        recipientId: contact.publicKey,
        sharedSecret: sharedSecret,
      );

      // Refresh messages
      ref.invalidate(messagesProvider(widget.contactId));

      // Scroll to bottom
      if (_scrollController.hasClients) {
        _scrollController.animateTo(
          0,
          duration: const Duration(milliseconds: 300),
          curve: Curves.easeOut,
        );
      }
    } catch (e) {
      _showError('Failed to send message: ${e.toString()}');
      // Restore message text on error
      _messageController.text = messageText;
    }
  }

  void _showError(String message) {
    if (mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(message),
          backgroundColor: Theme.of(context).colorScheme.error,
        ),
      );
    }
  }

  @override
  Widget build(BuildContext context) {
    final contactAsync = ref.watch(contactProvider(widget.contactId));
    final messagesAsync = ref.watch(messagesProvider(widget.contactId));

    return contactAsync.when(
      data: (contact) {
        if (contact == null) {
          return Scaffold(
            appBar: AppBar(title: const Text('Contact Not Found')),
            body: const Center(child: Text('Contact not found')),
          );
        }

        return Scaffold(
          appBar: AppBar(
            title: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(contact.name),
                Text(
                  contact.verified ? 'Verified' : 'Unverified',
                  style: Theme.of(context).textTheme.bodySmall,
                ),
              ],
            ),
            actions: [
              // Refresh button
              IconButton(
                icon: const Icon(Icons.refresh),
                onPressed: () {
                  final refresh = ref.read(refreshContactMessagesProvider(widget.contactId));
                  refresh();
                },
                tooltip: 'Refresh',
              ),
              IconButton(
                icon: const Icon(Icons.shield_outlined),
                onPressed: () => _showSafetyNumber(context),
                tooltip: 'Safety Number',
              ),
              IconButton(
                icon: const Icon(Icons.more_vert),
                onPressed: () => _showChatOptions(context),
              ),
            ],
          ),
      body: Column(
        children: [
          // Security banner
          Container(
            padding: const EdgeInsets.all(8),
            color: Theme.of(context).colorScheme.primaryContainer,
            child: Row(
              children: [
                Icon(
                  Icons.lock,
                  size: 16,
                  color: Theme.of(context).colorScheme.onPrimaryContainer,
                ),
                const SizedBox(width: 8),
                Expanded(
                  child: Text(
                    'End-to-end encrypted via Veilid',
                    style: Theme.of(context).textTheme.bodySmall?.copyWith(
                          color: Theme.of(context).colorScheme.onPrimaryContainer,
                        ),
                  ),
                ),
              ],
            ),
          ),

          // Messages list
          Expanded(
            child: messagesAsync.when(
              data: (messages) {
                if (messages.isEmpty) {
                  return Center(
                    child: Column(
                      mainAxisAlignment: MainAxisAlignment.center,
                      children: [
                        Icon(
                          Icons.message_outlined,
                          size: 64,
                          color: Theme.of(context).colorScheme.primary.withOpacity(0.5),
                        ),
                        const SizedBox(height: 16),
                        const Text('No messages yet'),
                        const SizedBox(height: 8),
                        const Text('Send a secure message to start chatting'),
                      ],
                    ),
                  );
                }

                return ListView.builder(
                  controller: _scrollController,
                  reverse: true,
                  padding: const EdgeInsets.all(16),
                  itemCount: messages.length,
                  itemBuilder: (context, index) {
                    final message = messages[index];
                    final myIdentity = ref.watch(currentIdentityProvider);
                    final isSent = message.senderId == (myIdentity?.publicKey ?? 'self');

                    return _MessageBubble(
                      message: message,
                      isSent: isSent,
                    );
                  },
                );
              },
              loading: () => const Center(child: CircularProgressIndicator()),
              error: (error, stack) => Center(child: Text('Error: $error')),
            ),
          ),

          // Message input
          Container(
            padding: const EdgeInsets.all(16),
            decoration: BoxDecoration(
              color: Theme.of(context).colorScheme.surface,
              border: Border(
                top: BorderSide(
                  color: Theme.of(context).dividerColor,
                ),
              ),
            ),
            child: Row(
              children: [
                Expanded(
                  child: TextField(
                    controller: _messageController,
                    decoration: InputDecoration(
                      hintText: 'Type a secure message...',
                      border: OutlineInputBorder(
                        borderRadius: BorderRadius.circular(24),
                      ),
                      contentPadding: const EdgeInsets.symmetric(
                        horizontal: 16,
                        vertical: 8,
                      ),
                    ),
                    maxLines: null,
                    textInputAction: TextInputAction.send,
                    onSubmitted: (_) => _handleSendMessage(),
                  ),
                ),
                const SizedBox(width: 8),
                IconButton.filled(
                  onPressed: _handleSendMessage,
                  icon: const Icon(Icons.send),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  void _showSafetyNumber(BuildContext context) async {
    final contactAsync = ref.read(contactProvider(widget.contactId));
    final contact = contactAsync.value;
    if (contact == null) return;

    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Safety Number'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Text(
              'Verify this safety number with your contact out-of-band (phone call, in person, etc.):',
            ),
            const SizedBox(height: 16),
            Text(
              contact.safetyNumber,
              style: Theme.of(context).textTheme.headlineMedium?.copyWith(
                    fontFamily: 'monospace',
                  ),
            ),
            const SizedBox(height: 16),
            const Text(
              'If the numbers match, you can be confident your connection is secure.',
            ),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('Close'),
          ),
          FilledButton(
            onPressed: () async {
              final verifyContact = ref.read(verifyContactProvider);
              await verifyContact(contact.id);
              if (context.mounted) {
                Navigator.of(context).pop();
                ScaffoldMessenger.of(context).showSnackBar(
                  const SnackBar(content: Text('Contact verified!')),
                );
              }
            },
            child: const Text('Mark as Verified'),
          ),
        ],
      ),
    );
  }

  void _showChatOptions(BuildContext context) {
    showModalBottomSheet(
      context: context,
      builder: (context) => SafeArea(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            ListTile(
              leading: const Icon(Icons.timer),
              title: const Text('Send Ephemeral Message'),
              onTap: () {
                Navigator.of(context).pop();
                // TODO: Show ephemeral message options
              },
            ),
            ListTile(
              leading: const Icon(Icons.delete_outline),
              title: const Text('Clear Chat History'),
              onTap: () {
                Navigator.of(context).pop();
                // TODO: Clear messages
              },
            ),
            ListTile(
              leading: const Icon(Icons.block),
              title: const Text('Block Contact'),
              textColor: Theme.of(context).colorScheme.error,
              iconColor: Theme.of(context).colorScheme.error,
              onTap: () {
                Navigator.of(context).pop();
                // TODO: Block contact
              },
            ),
          ],
        ),
      ),
    );
  }
}

class _MessageBubble extends StatelessWidget {
  final Message message;
  final bool isSent;

  const _MessageBubble({
    required this.message,
    required this.isSent,
  });

  @override
  Widget build(BuildContext context) {
    return Align(
      alignment: isSent ? Alignment.centerRight : Alignment.centerLeft,
      child: Container(
        margin: const EdgeInsets.only(bottom: 8),
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 10),
        decoration: BoxDecoration(
          color: isSent
              ? Theme.of(context).colorScheme.primaryContainer
              : Theme.of(context).colorScheme.secondaryContainer,
          borderRadius: BorderRadius.circular(16),
        ),
        constraints: BoxConstraints(
          maxWidth: MediaQuery.of(context).size.width * 0.75,
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(message.content),
            const SizedBox(height: 4),
            Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                Text(
                  _formatTime(message.timestamp),
                  style: Theme.of(context).textTheme.bodySmall,
                ),
                if (isSent) ...[
                  const SizedBox(width: 4),
                  Icon(
                    message.isRead
                        ? Icons.done_all
                        : message.isDelivered
                            ? Icons.done_all
                            : Icons.done,
                    size: 16,
                    color: message.isRead ? Colors.blue : null,
                  ),
                ],
              ],
            ),
          ],
        ),
      ),
    );
  }

  String _formatTime(DateTime? timestamp) {
    if (timestamp == null) return '';
    return '${timestamp.hour.toString().padLeft(2, '0')}:${timestamp.minute.toString().padLeft(2, '0')}';
  }
}
