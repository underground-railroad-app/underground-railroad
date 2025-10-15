import 'dart:async';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:underground_railroad/core/services/message_listener_service.dart';
import 'package:underground_railroad/shared/providers/app_providers.dart';

/// Auto-refresh messages periodically
class MessageRefreshNotifier extends StateNotifier<DateTime> {
  final MessageListenerService _listener;
  Timer? _refreshTimer;

  MessageRefreshNotifier(this._listener) : super(DateTime.now()) {
    _startAutoRefresh();
  }

  void _startAutoRefresh() {
    // Refresh every 10 seconds
    _refreshTimer = Timer.periodic(const Duration(seconds: 10), (_) {
      _listener.checkNow();
      state = DateTime.now(); // Trigger rebuild
    });
  }

  void refreshNow() {
    _listener.checkNow();
    state = DateTime.now();
  }

  @override
  void dispose() {
    _refreshTimer?.cancel();
    super.dispose();
  }
}

final messageRefreshProvider = StateNotifierProvider<MessageRefreshNotifier, DateTime>((ref) {
  final listener = ref.watch(messageListenerServiceProvider);
  return MessageRefreshNotifier(listener);
});

/// Refresh messages for a specific contact
final refreshContactMessagesProvider = Provider.family<void Function(), String>((ref, contactId) {
  return () {
    ref.invalidate(messagesProvider(contactId));
    ref.read(messageRefreshProvider.notifier).refreshNow();
  };
});
