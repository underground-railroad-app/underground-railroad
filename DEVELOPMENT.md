# Underground Railroad - Development Guide

## Project Status

**Current Phase:** Beta - Core features implemented, network integration in progress

This project implements a secure, encrypted assistance coordination app for people fleeing persecution—a modern Underground Railroad.

**Reality Check:** This is a **Flutter mobile/desktop app**, not a CLI tool. Core encryption and storage are production-ready. Veilid anonymous networking connects on desktop but DHT features are in development.

## What's Been Built

### Core Architecture (Rust)

#### 1. **Assistance Coordination** (`core/src/assistance/`)
The heart of the application - life-saving features:

- **Emergency Requests** (`emergency.rs`): ✅ Data structures complete
  - ✅ Priority scoring system
  - 🔄 Multi-hop network propagation (in development)
  - ✅ Need matching (shelter, transport, medical, etc.)

- **Safe House Registry** (`safe_house.rs`): ✅ Data structures complete
  - ✅ Capacity management
  - ✅ Capability matching
  - ✅ Trust-based verification

- **Transportation Network** (`transportation.rs`): ✅ Data structures complete
  - ✅ Driver/passenger matching
  - ✅ Special requirements (wheelchair, children, etc.)
  - ✅ Route flexibility

- **Intelligence Reports** (`intelligence.rs`): ✅ Data structures complete
  - ✅ Danger warnings (checkpoints, raids, etc.)
  - ✅ Safe route confirmation
  - ⚠️ Multi-source verification (basic implementation)

#### 2. **Trust System** (`core/src/trust/`)
Web of trust for decentralized verification:

- **Contact Management** (`contact.rs`): ✅ Fully implemented
  - ✅ Multiple trust levels
  - ✅ Capability tracking
  - ✅ Fingerprint verification

- **Verification** (`verification.rs`): ✅ Framework ready
  - ✅ In-person verification checklist
  - ⚠️ Verification workflows (basic)

- **Trust Graph** (`graph.rs`): ✅ Algorithms implemented
  - ✅ BFS path finding
  - ✅ Trust strength calculation
  - ⚠️ Network statistics (basic)

#### 3. **Encrypted Storage** (`core/src/storage/`)
SQLCipher database with encryption at rest:

- **Schema** (`schema.rs`): ✅ Complete
  - ✅ Identity, contacts, safe houses
  - ✅ Transportation, emergencies, intelligence
  - ✅ Messages, trust relationships

- **Database** (`database.rs`): ✅ Fully functional
  - ⚠️ Secure backup/restore (stubbed with TODO)
  - ✅ Secure deletion (multi-pass overwrite)
  - ✅ Transaction support
  - ✅ WAL mode, auto-vacuum

- **Repository Pattern** (`storage/repository/`): ✅ Complete
  - ✅ Contacts, messages, emergencies, safe houses
  - ✅ CRUD operations with proper error handling

#### 4. **Cryptography** (`core/src/crypto/` & `messaging/encryption.rs`)
Production-grade encryption:

- **Key Derivation** (`keys.rs`): ✅ Complete
  - ✅ Argon2id password hashing (64MB, 3 iterations, GPU-resistant)
  - ✅ HKDF-SHA512 key hierarchy
  - ✅ Zeroized memory (keys cleared on drop)
  - ✅ Separate derived keys: identity_seed, encryption_seed, storage_key

- **Message Encryption** (`messaging/encryption.rs`): ✅ Complete
  - ✅ Hybrid post-quantum: X25519 + Kyber1024 (NIST Level 5)
  - ✅ ChaCha20-Poly1305 authenticated encryption
  - ✅ Ephemeral keys for forward secrecy
  - ✅ Legacy X25519-only mode for compatibility
  - ⚠️ Note: Kyber keygen not deterministic (line 216 TODO)

#### 5. **Core Types** (`core/src/types.rs`)
✅ Foundational types implemented:

- ✅ **Coarse Timestamps**: 5-minute rounding (prevents timing attacks)
- ✅ **Coarse Regions**: Approximate locations (privacy-preserving)
- ✅ **Fingerprints**: Human-readable verification words (BIP39-based)
- ✅ **Secure Bytes**: Auto-zeroized sensitive data
- ✅ **Trust Levels**: Graduated trust system (Unknown, Weak, Medium, Strong, Verified)

#### 6. **Veilid Client** (`veilid_client/`)
⚠️ Partial implementation:

- ✅ Client initialization and state management
- ✅ Network attach/detach
- ✅ Routing context creation
- ✅ Works on desktop (macOS confirmed)
- 🔄 Mobile support (Android/iOS in progress)
- ❌ DHT mailboxes (not implemented)
- ⚠️ receive_message() returns None (TODO at line 249)

#### 7. **Messaging** (`messaging/`)
✅ Encryption complete, relay in development:

- ✅ Hybrid PQ encryption (X25519+Kyber1024)
- ✅ Message protocol and storage
- ✅ Conversation management
- ⚠️ File-based relay (temporary for local testing)
- 🔄 Veilid DHT mailboxes (in development)

### Security Features Implemented

✅ **Encryption at Rest**
- SQLCipher with AES-256
- Hardware-backed key derivation (ready for Secure Enclave/StrongBox/TPM)
- No plaintext on disk ever

✅ **Memory Security**
- Zeroization on drop (keys cleared from RAM)
- Secure types throughout

✅ **Metadata Protection**
- Coarse timestamps (5min intervals)
- Coarse locations (regions not addresses)
- No exact timing or location data

✅ **Trust-Based Access**
- Graduated trust levels
- Activity only visible to trusted contacts
- Web of trust graph for verification

✅ **Privacy by Design**
- Minimal metadata
- Obfuscated access patterns
- Secure deletion support

## Project Structure

```
underground-railroad/
├── Cargo.toml              # Workspace configuration
├── README.md               # User-facing documentation
├── DEVELOPMENT.md          # This file
│
├── core/                   # Core Rust library
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs          # Library root
│       ├── error.rs        # Error types
│       ├── types.rs        # Core types
│       │
│       ├── assistance/     # Life-saving coordination ✅
│       │   ├── mod.rs
│       │   ├── emergency.rs
│       │   ├── safe_house.rs
│       │   ├── transportation.rs
│       │   └── intelligence.rs
│       │
│       ├── trust/          # Web of trust ✅
│       │   ├── mod.rs
│       │   ├── contact.rs
│       │   ├── verification.rs
│       │   └── graph.rs
│       │
│       ├── storage/        # Encrypted database ✅
│       │   ├── mod.rs
│       │   ├── schema.rs
│       │   ├── database.rs
│       │   └── migrations.rs
│       │
│       ├── crypto/         # Cryptography ✅
│       │   ├── mod.rs
│       │   └── keys.rs
│       │
│       ├── identity/       # TODO: Multiple personas
│       │   └── mod.rs
│       │
│       └── veilid_client/  # TODO: Anonymous networking
│           └── mod.rs
│
└── cli/                    # Command-line interface
    ├── Cargo.toml
    └── src/
        └── main.rs         # CLI stub
```

## What's Implemented

### ✅ Phase 1: Foundation (COMPLETE)

1. **Identity Module** (`core/src/identity/`) - ✅ Complete
   - ✅ Ed25519 signing keys
   - ✅ X25519 + Kyber1024 hybrid encryption keys
   - ✅ Fingerprint generation (BIP39 words)
   - ✅ QR code generation/scanning
   - ⚠️ Single persona only (multiple personas planned)

2. **Cryptography** - ✅ Production-ready
   - ✅ Hybrid post-quantum encryption (X25519+Kyber1024)
   - ✅ ChaCha20-Poly1305 authenticated encryption
   - ✅ Argon2id password hashing
   - ✅ HKDF-SHA512 key derivation
   - ⚠️ Dilithium signatures (defined but not integrated)

3. **Database & Storage** - ✅ Complete
   - ✅ SQLCipher (AES-256)
   - ✅ Repository pattern (contacts, messages, emergencies, safe houses)
   - ✅ Migrations system
   - ✅ Secure deletion

4. **Flutter UI** - ✅ Complete
   - ✅ Material Design 3 interface
   - ✅ QR code scanning
   - ✅ Contact management
   - ✅ Encrypted messaging UI
   - ✅ Emergency/safe house forms
   - ✅ Dark mode support

### 🔄 Phase 2: Network Integration (IN PROGRESS)

1. **Veilid Integration**
   - ✅ Veilid client (desktop - macOS confirmed)
   - ✅ VeilidService with two-step initialization
   - 🔄 Mobile Veilid (Android/iOS in testing)
   - ❌ DHT mailboxes (not implemented)
   - ❌ Message routing via Veilid (uses file relay currently)

2. **Message Relay**
   - ✅ Encryption working (hybrid PQ)
   - ✅ File-based relay (testing/demo)
   - 🔄 Veilid DHT mailboxes (in development)
   - 🔄 Background polling service

3. **Broadcasting**
   - ❌ Emergency broadcasting (TODOat ffi/src/api.rs:211)
   - ❌ Safe house announcements (TODO at ffi/src/api.rs:269)
   - ❌ DHT record propagation

### 🔄 Phase 3: Platform Completion (IN PROGRESS)

1. **Platform Testing**
   - ✅ macOS: Fully tested
   - ✅ Android: Tested (offline mode)
   - ⚠️ iOS: Build ready, untested
   - ⚠️ Windows: Build script ready, untested
   - ⚠️ Linux: Build script ready, untested

2. **Mobile Features**
   - ✅ All UI implemented
   - ✅ Offline functionality complete
   - 🔄 Veilid networking
   - ⚠️ Biometric unlock (not implemented)
   - ⚠️ Background services (not implemented)

### 📋 Phase 4: Advanced Features (PLANNED)

1. **Traffic Obfuscation**
   - Fixed cell sizes
   - Cover traffic
   - Timing randomization

2. **Hardware Security**
   - Secure Enclave (iOS/macOS)
   - StrongBox (Android)
   - TPM (Windows/Linux)

3. **Additional Features**
   - Multiple personas
   - Deterministic Kyber keygen
   - i18n (internationalization)
   - Accessibility improvements

## Building the Project

**This is a Flutter app, not a CLI tool.**

```bash
# Build Rust FFI library
cargo build --release -p underground-railroad-ffi

# Or use platform-specific scripts
./build_and_bundle.sh    # macOS
./build_android.sh       # Android
./build_ios.sh           # iOS

# Run the Flutter app
cd mobile
flutter run -d macos     # or android, ios, etc.
```

**See [BUILD.md](BUILD.md) for complete instructions.**

## Testing Strategy

### Unit Tests
Each module has comprehensive unit tests:
- `core/src/types.rs`: Type behavior
- `core/src/assistance/*`: Assistance logic
- `core/src/trust/*`: Trust graph algorithms
- `core/src/storage/*`: Database operations
- `core/src/crypto/*`: Key derivation

### Integration Tests (TODO)
- End-to-end emergency flow
- Trust graph propagation
- Database encryption/decryption
- Message routing

### Security Tests (TODO)
- Timing attack resistance
- Metadata leakage prevention
- Key zeroization verification
- Secure deletion verification

## Dependencies

### Core
- `veilid-core`: Anonymous P2P networking
- `rusqlite` + `sqlcipher`: Encrypted database
- `argon2`: Password hashing
- `chacha20poly1305`: Symmetric encryption
- `ed25519-dalek`: Signing keys
- `x25519-dalek`: Encryption keys
- `zeroize`: Secure memory clearing

### Post-Quantum (Already Included)
- ✅ `pqcrypto-kyber`: Kyber1024 KEM (implemented)
- ⚠️ `pqcrypto-dilithium`: Dilithium signatures (defined, not integrated)

### Flutter
- `flutter`: Cross-platform UI framework
- `veilid`: Anonymous networking (v0.4.8)
- `mobile_scanner`: QR code scanning
- `flutter_secure_storage`: Encrypted key storage

## Security Considerations

### Current Protections
1. Encryption at rest (SQLCipher)
2. Memory zeroization (sensitive data cleared)
3. Coarse timestamps (timing attack resistant)
4. Coarse locations (privacy-preserving)
5. Trust-based access control

### Planned Protections
1. Hardware-backed keys (Secure Enclave/StrongBox/TPM)
2. Dilithium post-quantum signatures (Kyber1024 already implemented)
3. Traffic obfuscation (padding, timing)
4. ORAM patterns (oblivious database access)
5. Plausible deniability (hidden volumes)
6. Deterministic Kyber key generation (currently random)

### Threat Model
**Adversary Capabilities:**
- Nation-state resources
- Network surveillance (global)
- Device seizure + forensics
- Rubber-hose cryptanalysis (coercion)

**Our Defenses:**
- Multi-layer encryption
- Anonymous networking (Veilid)
- Minimal metadata
- Secure deletion
- Compartmentalization (personas)
- Panic wipe capability

## Design Principles

1. **Life-Saving First**: Features directly map to Underground Railroad functions
2. **Security Enables Mission**: Strong security supports (not obscures) coordination
3. **Usable Under Stress**: Dead-simple UI for emergencies
4. **Universal Access**: i18n and a11y from day one
5. **Zero Trust**: No central authority, no single point of failure
6. **Privacy by Design**: Minimal metadata, coarse granularity
7. **Defense in Depth**: Multiple independent security layers

## License

GPL-3.0-or-later - This software must remain free and open source.

## Contributing

See `CONTRIBUTING.md` (TODO)

## Security

Report vulnerabilities to: underground_railroad_app@proton.me
