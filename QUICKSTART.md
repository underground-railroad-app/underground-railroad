# 🚀 Underground Railroad - Quick Start

**Get up and running in 5 minutes!**

## Prerequisites

- **Flutter SDK 3.27+**: `flutter --version`
- **Rust 1.75+**: `rustc --version`
- **Git**: `git --version`

## Automated Setup (Recommended)

```bash
# Clone or navigate to project
cd underground-railroad

# Run setup script (does everything)
./setup.sh

# Run the app
flutter run -d macos  # or android, ios, linux, windows
```

## Manual Setup

If the script doesn't work, run these commands:

```bash
# 1. Install bridge codegen
cargo install flutter_rust_bridge_codegen

# 2. Test Rust crypto
cd rust && cargo test && cd ..

# 3. Generate bridge
flutter_rust_bridge_codegen generate

# 4. Get Flutter dependencies
flutter pub get

# 5. Generate Dart code
dart run build_runner build --delete-conflicting-outputs

# 6. Build Rust library
cd rust && cargo build && cd ..

# 7. Run!
flutter run -d macos
```

## First Launch

### 1. Set Up PIN
- Choose a secure PIN (6+ digits)
- Optionally set a duress PIN
- Duress PIN opens decoy account

### 2. Add a Contact
- Tap "+" button
- Enter contact details:
  - Name
  - Veilid Route (starts with VLD1:route:...)
  - Public Key (starts with VLD1:pub:...)
- Tap "Add"

### 3. Send a Message
- Tap on contact
- Type message
- Tap send
- Message is encrypted and sent!

## Testing the Security

### E2E Encryption
```bash
# Check database is encrypted
cd ~/Library/Application\ Support/com.example.undergroundRailroad/
sqlite3 underground_railroad.db ".schema"
# Should fail without password - database is encrypted!
```

### Duress Mode
1. Set up duress PIN during initial setup
2. Close and reopen app
3. Enter duress PIN
4. Should see decoy contacts (fake data)
5. Real contacts are hidden

### Veilid Network
- Check console logs for Veilid initialization
- Messages are routed through private routes
- No direct connection between sender/receiver

## Common Issues

### Bridge Generation Fails
```bash
# Update codegen
cargo install --force flutter_rust_bridge_codegen

# Try again
flutter_rust_bridge_codegen generate
```

### Rust Build Fails
```bash
# Clean and rebuild
cd rust
cargo clean
cargo build
cd ..
```

### Flutter Errors
```bash
# Clean Flutter
flutter clean
flutter pub get
dart run build_runner build --delete-conflicting-outputs
```

### "File not found" Errors
```bash
# Make sure bridge is generated
ls lib/generated/bridge.dart

# Make sure models are generated
ls lib/shared/models/*.g.dart
```

## Project Structure

```
underground-railroad/
├── lib/
│   ├── main.dart              # App entry
│   ├── core/                  # Core services
│   │   ├── crypto/            # Encryption
│   │   ├── storage/           # Databases
│   │   ├── security/          # Auth & duress
│   │   ├── veilid/            # Network
│   │   └── services/          # Background services
│   ├── features/              # App features
│   │   ├── auth/              # Authentication
│   │   ├── contacts/          # Contact management
│   │   └── messaging/         # Encrypted messaging
│   └── shared/                # Shared code
│       ├── models/            # Data models
│       └── providers/         # Riverpod providers
└── rust/                      # Rust crypto core
    └── src/
        ├── crypto.rs          # Crypto primitives
        ├── veilid_manager.rs  # Veilid integration
        └── api.rs             # Flutter bridge
```

## Key Features

### 🔐 Security
- **ChaCha20-Poly1305** encryption
- **Argon2id** key derivation
- **Blake3** hashing
- **Veilid** anonymous routing
- **SQLCipher** encrypted storage

### 🎭 Privacy
- **End-to-end encryption** (E2E)
- **Anonymous routing** (no metadata)
- **Plausible deniability** (duress mode)
- **Encrypted storage** (at rest)
- **Zero-knowledge** architecture

### 🚀 Features
- **Secure messaging** (1-to-1)
- **Contact verification** (safety numbers)
- **Ephemeral messages** (auto-delete)
- **Background sync** (auto-refresh)
- **Push notifications** (encrypted)
- **Multi-platform** (5 platforms)

## Next Steps

### Explore Features
1. **Verify a contact**: View safety number, compare out-of-band
2. **Send ephemeral message**: Auto-deletes after time
3. **Test duress mode**: Use duress PIN to access decoy
4. **Check encryption**: Verify database is encrypted

### Read Documentation
- **READY_TO_RUN.md**: Detailed setup guide
- **MESSAGING_COMPLETE.md**: Messaging architecture
- **BUILD_GUIDE.md**: Platform-specific builds
- **FINAL_STATUS.md**: Project status

### Development
- **Add features**: See STATUS.md for remaining work
- **Run tests**: `flutter test` (once written)
- **Debug**: Check console for detailed logs
- **Contribute**: See CONTRIBUTING.md (TBD)

## Support

### Documentation
- 📖 **READY_TO_RUN.md** - Complete setup guide
- 📖 **MESSAGING_IMPLEMENTATION.md** - Architecture details
- 📖 **BUILD_GUIDE.md** - Platform builds
- 📖 **STATUS.md** - Current status

### Debugging
```bash
# Verbose Flutter logs
flutter run -v

# Rust logs
cd rust && RUST_LOG=debug cargo test

# Check generated files
ls -la lib/generated/
ls -la lib/shared/models/*.g.dart
```

## Quick Commands

```bash
# Run on different platforms
flutter run -d macos
flutter run -d android
flutter run -d ios
flutter run -d linux
flutter run -d windows

# Hot reload (during development)
# Press 'r' in terminal to reload
# Press 'R' to hot restart

# Clean everything
flutter clean
cd rust && cargo clean && cd ..

# Rebuild from scratch
./setup.sh
flutter run
```

## Success Indicators

✅ **App launches** without errors
✅ **PIN setup** works
✅ **Contacts screen** shows
✅ **Can add contact** manually
✅ **Chat screen** opens
✅ **Messages send** (check console)
✅ **Database encrypted** (can't open with sqlite3)
✅ **Duress mode** switches databases

## Resources

- **GitHub Issues**: Report bugs
- **Documentation**: See `*.md` files
- **Veilid Docs**: https://veilid.com
- **Flutter Docs**: https://flutter.dev

---

**Ready to build the most secure messenger on the planet!** 🔐🚀

Get started: `./setup.sh`
