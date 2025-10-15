import 'package:sqflite_sqlcipher/sqflite.dart';
import 'package:underground_railroad/core/crypto/message_crypto_service.dart';
import 'package:underground_railroad/core/storage/database_service.dart';
import 'package:underground_railroad/core/veilid/veilid_service.dart';
import 'package:underground_railroad/shared/models/contact.dart';
import 'package:uuid/uuid.dart';

class ContactRepository {
  final DatabaseService _db;
  final VeilidService _veilid;
  final MessageCryptoService _crypto;

  ContactRepository({
    required DatabaseService db,
    required VeilidService veilid,
    required MessageCryptoService crypto,
  })  : _db = db,
        _veilid = veilid,
        _crypto = crypto;

  /// Get all contacts
  Future<List<Contact>> getContacts() async {
    final db = _db.database;
    final results = await db.query(
      'contacts',
      orderBy: 'name ASC',
    );

    return results.map((map) => _contactFromMap(map)).toList();
  }

  /// Get contact by ID
  Future<Contact?> getContact(String id) async {
    final db = _db.database;
    final results = await db.query(
      'contacts',
      where: 'id = ?',
      whereArgs: [id],
      limit: 1,
    );

    if (results.isEmpty) return null;
    return _contactFromMap(results.first);
  }

  /// Add new contact
  Future<Contact> addContact({
    required String name,
    required String veilidRoute,
    required String publicKey,
    String? myPrivateKey,
  }) async {
    final now = DateTime.now();

    // Generate safety number for verification
    final myPublicKey = myPrivateKey ?? 'self'; // TODO: Get from Veilid identity
    final safetyNumber = await _crypto.generateSafetyNumber(
      myPublicKey: myPublicKey,
      theirPublicKey: publicKey,
    );

    final contact = Contact(
      id: const Uuid().v4(),
      name: name,
      veilidRoute: veilidRoute,
      publicKey: publicKey,
      safetyNumber: safetyNumber,
      verified: false,
      trustLevel: 0,
      createdAt: now,
      updatedAt: now,
    );

    final db = _db.database;
    await db.insert('contacts', _contactToMap(contact));

    return contact;
  }

  /// Update contact
  Future<void> updateContact(Contact contact) async {
    final db = _db.database;
    await db.update(
      'contacts',
      _contactToMap(contact.copyWith(updatedAt: DateTime.now())),
      where: 'id = ?',
      whereArgs: [contact.id],
    );
  }

  /// Delete contact
  Future<void> deleteContact(String id) async {
    final db = _db.database;
    await db.delete(
      'contacts',
      where: 'id = ?',
      whereArgs: [id],
    );
  }

  /// Mark contact as verified
  Future<void> verifyContact(String id) async {
    final db = _db.database;
    await db.update(
      'contacts',
      {
        'verified': 1,
        'updated_at': DateTime.now().millisecondsSinceEpoch,
      },
      where: 'id = ?',
      whereArgs: [id],
    );
  }

  /// Update trust level (0-3)
  Future<void> updateTrustLevel(String id, int level) async {
    if (level < 0 || level > 3) {
      throw ArgumentError('Trust level must be between 0 and 3');
    }

    final db = _db.database;
    await db.update(
      'contacts',
      {
        'trust_level': level,
        'updated_at': DateTime.now().millisecondsSinceEpoch,
      },
      where: 'id = ?',
      whereArgs: [id],
    );
  }

  /// Exchange contact info via QR code / Veilid DHT
  /// Returns contact exchange data for sharing
  Future<ContactExchangeData> createContactExchange({
    required String myName,
    required String myPublicKey,
    required String myVeilidRoute,
  }) async {
    // Store in DHT for retrieval
    final exchangeId = const Uuid().v4();

    // TODO: Store in Veilid DHT
    // await _veilid.setDHTValue(exchangeId, exchangeData);

    return ContactExchangeData(
      exchangeId: exchangeId,
      name: myName,
      publicKey: myPublicKey,
      veilidRoute: myVeilidRoute,
      timestamp: DateTime.now(),
    );
  }

  /// Retrieve contact from exchange data
  Future<Contact?> importContactFromExchange(String exchangeId) async {
    // TODO: Retrieve from Veilid DHT
    // final data = await _veilid.getDHTValue(exchangeId);
    // if (data == null) return null;

    // Parse and create contact
    // For now, return null
    return null;
  }

  Contact _contactFromMap(Map<String, dynamic> map) {
    return Contact(
      id: map['id'] as String,
      name: map['name'] as String,
      veilidRoute: map['veilid_route'] as String,
      publicKey: map['public_key'] as String,
      safetyNumber: map['safety_number'] as String,
      verified: (map['verified'] as int) == 1,
      trustLevel: map['trust_level'] as int,
      createdAt: DateTime.fromMillisecondsSinceEpoch(map['created_at'] as int),
      updatedAt: DateTime.fromMillisecondsSinceEpoch(map['updated_at'] as int),
    );
  }

  Map<String, dynamic> _contactToMap(Contact contact) {
    return {
      'id': contact.id,
      'name': contact.name,
      'veilid_route': contact.veilidRoute,
      'public_key': contact.publicKey,
      'safety_number': contact.safetyNumber,
      'verified': contact.verified ? 1 : 0,
      'trust_level': contact.trustLevel,
      'created_at': contact.createdAt.millisecondsSinceEpoch,
      'updated_at': contact.updatedAt.millisecondsSinceEpoch,
    };
  }
}

/// Contact exchange data for QR codes / DHT sharing
class ContactExchangeData {
  final String exchangeId;
  final String name;
  final String publicKey;
  final String veilidRoute;
  final DateTime timestamp;

  ContactExchangeData({
    required this.exchangeId,
    required this.name,
    required this.publicKey,
    required this.veilidRoute,
    required this.timestamp,
  });

  Map<String, dynamic> toJson() => {
        'exchangeId': exchangeId,
        'name': name,
        'publicKey': publicKey,
        'veilidRoute': veilidRoute,
        'timestamp': timestamp.toIso8601String(),
      };

  factory ContactExchangeData.fromJson(Map<String, dynamic> json) {
    return ContactExchangeData(
      exchangeId: json['exchangeId'] as String,
      name: json['name'] as String,
      publicKey: json['publicKey'] as String,
      veilidRoute: json['veilidRoute'] as String,
      timestamp: DateTime.parse(json['timestamp'] as String),
    );
  }
}
