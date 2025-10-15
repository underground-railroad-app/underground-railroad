import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:underground_railroad/shared/models/contact.dart';
import 'package:underground_railroad/shared/providers/app_providers.dart';

/// Add contact action
final addContactProvider = Provider<Future<Contact> Function({
  required String name,
  required String veilidRoute,
  required String publicKey,
  String? myPrivateKey,
})>((ref) {
  final repository = ref.watch(contactRepositoryProvider);

  return ({
    required String name,
    required String veilidRoute,
    required String publicKey,
    String? myPrivateKey,
  }) async {
    final contact = await repository.addContact(
      name: name,
      veilidRoute: veilidRoute,
      publicKey: publicKey,
      myPrivateKey: myPrivateKey,
    );

    // Invalidate contacts list to refresh
    ref.invalidate(contactsProvider);

    return contact;
  };
});

/// Update contact action
final updateContactProvider = Provider<Future<void> Function(Contact)>((ref) {
  final repository = ref.watch(contactRepositoryProvider);

  return (contact) async {
    await repository.updateContact(contact);
    ref.invalidate(contactsProvider);
    ref.invalidate(contactProvider(contact.id));
  };
});

/// Delete contact action
final deleteContactProvider = Provider<Future<void> Function(String)>((ref) {
  final repository = ref.watch(contactRepositoryProvider);

  return (id) async {
    await repository.deleteContact(id);
    ref.invalidate(contactsProvider);
  };
});

/// Verify contact action
final verifyContactProvider = Provider<Future<void> Function(String)>((ref) {
  final repository = ref.watch(contactRepositoryProvider);

  return (id) async {
    await repository.verifyContact(id);
    ref.invalidate(contactsProvider);
    ref.invalidate(contactProvider(id));
  };
});

/// Update trust level action
final updateTrustLevelProvider = Provider<Future<void> Function(String, int)>((ref) {
  final repository = ref.watch(contactRepositoryProvider);

  return (id, level) async {
    await repository.updateTrustLevel(id, level);
    ref.invalidate(contactsProvider);
    ref.invalidate(contactProvider(id));
  };
});
