import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:underground_railroad/core/crypto/crypto_service.dart';
import 'package:underground_railroad/core/crypto/message_crypto_service.dart';
import 'package:underground_railroad/core/security/duress_manager.dart';
import 'package:underground_railroad/core/security/security_manager.dart';
import 'package:underground_railroad/core/services/message_listener_service.dart';
import 'package:underground_railroad/core/services/notification_service.dart';
import 'package:underground_railroad/core/storage/database_service.dart';
import 'package:underground_railroad/core/storage/secure_storage_service.dart';
import 'package:underground_railroad/core/veilid/veilid_service.dart';
import 'package:underground_railroad/features/contacts/data/contact_repository.dart';
import 'package:underground_railroad/features/messaging/data/message_repository.dart';
import 'package:underground_railroad/shared/models/contact.dart';
import 'package:underground_railroad/shared/models/message.dart';

// Export commonly used providers for convenience
export 'package:underground_railroad/core/services/message_listener_service.dart' show messageListenerServiceProvider;
export 'package:underground_railroad/core/services/notification_service.dart' show notificationServiceProvider;

/// Core service providers

final secureStorageProvider = Provider<SecureStorageService>((ref) {
  return SecureStorageService();
});

final cryptoServiceProvider = Provider<CryptoService>((ref) {
  return CryptoService();
});

final messageCryptoServiceProvider = Provider<MessageCryptoService>((ref) {
  final crypto = ref.watch(cryptoServiceProvider);
  return MessageCryptoService(crypto: crypto);
});

final databaseServiceProvider = Provider<DatabaseService>((ref) {
  return DatabaseService();
});

final veilidServiceProvider = Provider<VeilidService>((ref) {
  return VeilidService();
});

final securityManagerProvider = Provider<SecurityManager>((ref) {
  final secureStorage = ref.watch(secureStorageProvider);
  final crypto = ref.watch(cryptoServiceProvider);
  return SecurityManager(
    secureStorage: secureStorage,
    crypto: crypto,
  );
});

final duressManagerProvider = Provider<DuressManager>((ref) {
  final databaseService = ref.watch(databaseServiceProvider);
  final secureStorage = ref.watch(secureStorageProvider);
  return DuressManager(
    databaseService: databaseService,
    secureStorage: secureStorage,
  );
});

/// Repository providers

final contactRepositoryProvider = Provider<ContactRepository>((ref) {
  final db = ref.watch(databaseServiceProvider);
  final veilid = ref.watch(veilidServiceProvider);
  final crypto = ref.watch(messageCryptoServiceProvider);
  return ContactRepository(
    db: db,
    veilid: veilid,
    crypto: crypto,
  );
});

final messageRepositoryProvider = Provider<MessageRepository>((ref) {
  final db = ref.watch(databaseServiceProvider);
  final veilid = ref.watch(veilidServiceProvider);
  final crypto = ref.watch(messageCryptoServiceProvider);
  return MessageRepository(
    db: db,
    veilid: veilid,
    crypto: crypto,
  );
});

/// Data providers

// Contacts list
final contactsProvider = FutureProvider<List<Contact>>((ref) async {
  final repository = ref.watch(contactRepositoryProvider);
  return await repository.getContacts();
});

// Single contact
final contactProvider = FutureProvider.family<Contact?, String>((ref, id) async {
  final repository = ref.watch(contactRepositoryProvider);
  return await repository.getContact(id);
});

// Messages for a contact
final messagesProvider = FutureProvider.family<List<Message>, String>((ref, contactId) async {
  final repository = ref.watch(messageRepositoryProvider);
  return await repository.getMessagesForContact(contactId);
});

// Unread count for a contact
final unreadCountProvider = FutureProvider.family<int, String?>((ref, contactId) async {
  final repository = ref.watch(messageRepositoryProvider);
  return await repository.getUnreadCount(contactId: contactId);
});

// Total unread count
final totalUnreadCountProvider = FutureProvider<int>((ref) async {
  final repository = ref.watch(messageRepositoryProvider);
  return await repository.getUnreadCount();
});

/// State notifier providers

// Current user identity
final currentIdentityProvider = StateProvider<VeilidIdentity?>((ref) => null);

// Authentication state
enum AuthState {
  initial,
  unauthenticated,
  authenticated,
  duressMode,
}

final authStateProvider = StateNotifierProvider<AuthStateNotifier, AuthState>((ref) {
  return AuthStateNotifier();
});

class AuthStateNotifier extends StateNotifier<AuthState> {
  AuthStateNotifier() : super(AuthState.initial);

  void setAuthenticated({bool isDuressMode = false}) {
    state = isDuressMode ? AuthState.duressMode : AuthState.authenticated;
  }

  void setUnauthenticated() {
    state = AuthState.unauthenticated;
  }

  void logout() {
    state = AuthState.unauthenticated;
  }
}

// Selected contact for chat
final selectedContactProvider = StateProvider<String?>((ref) => null);
