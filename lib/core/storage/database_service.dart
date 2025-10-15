import 'package:path/path.dart';
import 'package:path_provider/path_provider.dart';
import 'package:sqflite_sqlcipher/sqflite.dart';
import 'package:underground_railroad/core/constants/app_constants.dart';

/// Encrypted database service using SQLCipher
/// Supports dual databases (real and decoy) for plausible deniability
class DatabaseService {
  Database? _database;
  Database? _decoyDatabase;
  bool _isDecoyMode = false;

  /// Get the active database (real or decoy based on mode)
  Database get database {
    if (_isDecoyMode) {
      if (_decoyDatabase == null) {
        throw StateError('Decoy database not initialized');
      }
      return _decoyDatabase!;
    } else {
      if (_database == null) {
        throw StateError('Database not initialized');
      }
      return _database!;
    }
  }

  /// Check if in decoy mode
  bool get isDecoyMode => _isDecoyMode;

  /// Initialize the real database
  Future<void> initializeRealDatabase(String password) async {
    final directory = await getApplicationDocumentsDirectory();
    final path = join(directory.path, AppConstants.dbName);

    _database = await openDatabase(
      path,
      version: AppConstants.dbVersion,
      password: password,
      onCreate: _createSchema,
      onUpgrade: _upgradeSchema,
    );

    _isDecoyMode = false;
  }

  /// Initialize the decoy database
  Future<void> initializeDecoyDatabase(String password) async {
    final directory = await getApplicationDocumentsDirectory();
    final path = join(directory.path, AppConstants.decoyDbName);

    _decoyDatabase = await openDatabase(
      path,
      version: AppConstants.dbVersion,
      password: password,
      onCreate: _createSchema,
      onUpgrade: _upgradeSchema,
    );
  }

  /// Switch to decoy mode (for duress PIN)
  Future<void> switchToDecoyMode() async {
    if (_decoyDatabase == null) {
      throw StateError('Decoy database not initialized');
    }
    _isDecoyMode = true;
  }

  /// Switch to real mode
  Future<void> switchToRealMode() async {
    if (_database == null) {
      throw StateError('Real database not initialized');
    }
    _isDecoyMode = false;
  }

  /// Create database schema
  Future<void> _createSchema(Database db, int version) async {
    // Contacts table
    await db.execute('''
      CREATE TABLE contacts (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        veilid_route TEXT NOT NULL,
        public_key TEXT NOT NULL,
        safety_number TEXT NOT NULL,
        verified INTEGER NOT NULL DEFAULT 0,
        trust_level INTEGER NOT NULL DEFAULT 0,
        created_at INTEGER NOT NULL,
        updated_at INTEGER NOT NULL
      )
    ''');

    // Messages table
    await db.execute('''
      CREATE TABLE messages (
        id TEXT PRIMARY KEY,
        contact_id TEXT NOT NULL,
        content TEXT NOT NULL,
        sender_id TEXT NOT NULL,
        recipient_id TEXT NOT NULL,
        timestamp INTEGER NOT NULL,
        is_sent INTEGER NOT NULL DEFAULT 0,
        is_delivered INTEGER NOT NULL DEFAULT 0,
        is_read INTEGER NOT NULL DEFAULT 0,
        is_ephemeral INTEGER NOT NULL DEFAULT 0,
        ephemeral_duration INTEGER,
        message_type TEXT,
        created_at INTEGER NOT NULL,
        FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE
      )
    ''');

    // Alerts table
    await db.execute('''
      CREATE TABLE alerts (
        id TEXT PRIMARY KEY,
        type TEXT NOT NULL,
        title TEXT NOT NULL,
        message TEXT NOT NULL,
        priority INTEGER NOT NULL DEFAULT 0,
        location_lat REAL,
        location_lng REAL,
        location_obfuscated TEXT,
        timestamp INTEGER NOT NULL,
        expires_at INTEGER,
        created_at INTEGER NOT NULL
      )
    ''');

    // Settings table
    await db.execute('''
      CREATE TABLE settings (
        key TEXT PRIMARY KEY,
        value TEXT NOT NULL,
        updated_at INTEGER NOT NULL
      )
    ''');

    // Create indices
    await db.execute('CREATE INDEX idx_messages_contact ON messages(contact_id)');
    await db.execute('CREATE INDEX idx_messages_timestamp ON messages(timestamp)');
    await db.execute('CREATE INDEX idx_alerts_timestamp ON alerts(timestamp)');
  }

  /// Upgrade database schema
  Future<void> _upgradeSchema(Database db, int oldVersion, int newVersion) async {
    // Handle schema migrations here
    // For now, no migrations needed
  }

  /// Close databases
  Future<void> close() async {
    await _database?.close();
    await _decoyDatabase?.close();
    _database = null;
    _decoyDatabase = null;
  }

  /// Emergency wipe - delete database files
  Future<void> emergencyWipe({bool wipeDecoy = false}) async {
    await close();

    final directory = await getApplicationDocumentsDirectory();

    // Wipe real database
    final realPath = join(directory.path, AppConstants.dbName);
    final realFile = await databaseExists(realPath);
    if (realFile) {
      await deleteDatabase(realPath);
    }

    // Optionally wipe decoy (usually we keep decoy during panic)
    if (wipeDecoy) {
      final decoyPath = join(directory.path, AppConstants.decoyDbName);
      final decoyFile = await databaseExists(decoyPath);
      if (decoyFile) {
        await deleteDatabase(decoyPath);
      }
    }
  }
}
