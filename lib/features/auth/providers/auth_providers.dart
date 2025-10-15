import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:underground_railroad/core/crypto/crypto_service.dart';
import 'package:underground_railroad/core/security/duress_manager.dart';
import 'package:underground_railroad/core/security/security_manager.dart';
import 'package:underground_railroad/core/storage/database_service.dart';
import 'package:underground_railroad/core/storage/secure_storage_service.dart';

/// Provider for secure storage service
final secureStorageProvider = Provider<SecureStorageService>((ref) {
  return SecureStorageService();
});

/// Provider for crypto service
final cryptoServiceProvider = Provider<CryptoService>((ref) {
  return CryptoService();
});

/// Provider for database service
final databaseServiceProvider = Provider<DatabaseService>((ref) {
  return DatabaseService();
});

/// Provider for security manager
final securityManagerProvider = Provider<SecurityManager>((ref) {
  final secureStorage = ref.watch(secureStorageProvider);
  final crypto = ref.watch(cryptoServiceProvider);

  return SecurityManager(
    secureStorage: secureStorage,
    crypto: crypto,
  );
});

/// Provider for duress manager (override with actual instances)
final duressManagerProvider = Provider<DuressManager>((ref) {
  final databaseService = ref.watch(databaseServiceProvider);
  final secureStorage = ref.watch(secureStorageProvider);

  return DuressManager(
    databaseService: databaseService,
    secureStorage: secureStorage,
  );
});

/// Authentication state provider
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
