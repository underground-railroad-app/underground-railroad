import 'package:freezed_annotation/freezed_annotation.dart';

part 'contact.freezed.dart';
part 'contact.g.dart';

@freezed
class Contact with _$Contact {
  const factory Contact({
    required String id,
    required String name,
    required String veilidRoute,
    required String publicKey,
    required String safetyNumber,
    @Default(false) bool verified,
    @Default(0) int trustLevel,
    required DateTime createdAt,
    required DateTime updatedAt,
  }) = _Contact;

  factory Contact.fromJson(Map<String, dynamic> json) =>
      _$ContactFromJson(json);
}

/// Contact with encryption keys for E2E messaging
@freezed
class SecureContact with _$SecureContact {
  const factory SecureContact({
    required Contact contact,
    required String sharedSecret, // For E2E encryption
    String? ephemeralKey, // For forward secrecy (future Double Ratchet)
  }) = _SecureContact;

  factory SecureContact.fromJson(Map<String, dynamic> json) =>
      _$SecureContactFromJson(json);
}
