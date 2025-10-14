# Build Instructions - Underground Railroad

## Quick Start

### Prerequisites

**Install Rust:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**Install Flutter:**
Visit https://docs.flutter.dev/get-started/install

**Install Android NDK** (for Android builds):
- Open Android Studio
- SDK Manager → SDK Tools → NDK (version 27.0.12077973)

---

## Build Commands

### macOS
```bash
./build_and_bundle.sh
cd mobile && flutter run -d macos
```

### Android
```bash
./build_android.sh
cd mobile && flutter run -d android
```

### iOS
```bash
./build_ios.sh
cd mobile && flutter run -d ios
```

### Linux (on Linux machine)
```bash
./build_linux.sh
cd mobile && flutter run -d linux
```

### Windows (on Windows machine)
```bash
./build_windows.sh
cd mobile && flutter run -d windows
```

---

## Platform-Specific Notes

### Android

**Architectures built:**
- ARM64 (arm64-v8a) - Modern devices
- ARMv7 (armeabi-v7a) - Older devices
- x86_64 - Emulators
- x86 (i686) - Older emulators

**Build time:** ~10-15 minutes (first build), ~2 minutes (subsequent)

**Veilid status:** Mobile Veilid integration in progress. App fully functional offline.

### iOS

**Requirements:**
- macOS with Xcode
- iOS 15.6+

**Architectures:**
- ARM64 (device)
- ARM64 simulator (M1/M2 Macs)
- x86_64 simulator (Intel Macs)

### Desktop Platforms

**Veilid:** ✅ Fully working on macOS, Windows, Linux

**Data location:**
- macOS/Linux: `~/.underground-railroad/{user-id}/`
- Windows: `%USERPROFILE%\.underground-railroad\{user-id}\`

---

## Build System

### What the Scripts Do

**build_and_bundle.sh** - Universal build script
- Detects platform from argument
- Builds Rust FFI library
- Copies to appropriate location

**build_android.sh** - Android-specific
- Builds for all 4 Android architectures
- Sets up NDK toolchain
- Copies .so files to `jniLibs/`

**build_ios.sh** - iOS-specific
- Builds for device + simulators
- Creates universal binary for simulators
- Copies to `ios/Frameworks/`

**build_linux.sh** / **build_windows.sh** - Platform-specific
- Best run on native platform
- Cross-compilation possible but complex

---

## Distribution Builds

### Release Builds

```bash
cd mobile

# iOS
flutter build ios --release

# Android
flutter build apk --release  # APK for sideloading
flutter build appbundle --release  # For Google Play

# macOS
flutter build macos --release

# Windows
flutter build windows --release

# Linux
flutter build linux --release
```

### APK Locations

After building:
- **Android APK:** `mobile/build/app/outputs/flutter-apk/app-release.apk`
- **Android Bundle:** `mobile/build/app/outputs/bundle/release/app-release.aab`
- **macOS:** `mobile/build/macos/Build/Products/Release/underground_railroad.app`
- **iOS:** `mobile/build/ios/iphoneos/Runner.app`

---

## Dependencies

### Veilid

The app uses the Veilid anonymous networking library:
- **Repository:** https://gitlab.com/veilid/veilid
- **Location:** Must be cloned to `../veilid` relative to project root
- **Version:** 0.4.8

**Setup:**
```bash
cd <parent-directory>
git clone https://gitlab.com/veilid/veilid.git
cd veilid/veilid-flutter
./setup_flutter.sh
```

### Rust Targets

**For Android:**
```bash
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi
rustup target add x86_64-linux-android
rustup target add i686-linux-android
```

**For iOS:**
```bash
rustup target add aarch64-apple-ios
rustup target add aarch64-apple-ios-sim
rustup target add x86_64-apple-ios
```

---

## Troubleshooting

### "FFI not loaded" Error

```bash
# Rebuild the FFI library
cargo build --release -p underground-railroad-ffi

# For Android
./build_android.sh

# Then run the app
cd mobile && flutter run -d <platform>
```

### "Database error: file is not a database"

Old corrupted database. Clear app data:

**Android:**
```bash
flutter run -d emulator-5554 --uninstall-first
```

**macOS:**
```bash
rm -rf ~/.underground-railroad
```

### Veilid Not Connecting on Mobile

This is expected. Desktop platforms (macOS/Windows/Linux) have full Veilid support. Mobile Veilid integration is in progress - app works fully offline.

### Build Fails

```bash
# Clean and rebuild
flutter clean
cargo clean
./build_and_bundle.sh <platform>
```

---

## Data Storage

### Locations

**macOS/Linux:**
```
~/.underground-railroad/
└── {user-id}/
    ├── salt              # Password salt
    ├── railroad.db       # Encrypted database
    ├── veilid/          # Veilid network data
    └── messages/        # Encrypted messages
```

**iOS/Android:**
```
{app-documents}/underground-railroad/
└── {user-id}/
    └── (same structure)
```

### Security

- All data encrypted with SQLCipher (AES-256)
- Password never stored (only salt)
- Keys derived from password using Argon2id
- User ID derived deterministically from password

---

## Development

### Hot Reload

```bash
# Terminal 1: Auto-rebuild Rust on changes
cargo watch -x 'build --release -p underground-railroad-ffi'

# Terminal 2: Run Flutter with hot reload
cd mobile
flutter run -d macos  # Press 'r' for hot reload
```

### Testing

```bash
# Run Rust tests
cargo test

# Run Flutter tests
cd mobile && flutter test
```

---

## Documentation

- **README.md** - This file
- **BUILD.md** - Complete build instructions
- **SECURITY.md** - Security model and threat analysis
- **DEVELOPMENT.md** - Developer guide
- **SESSION_PROGRESS.md** - Recent changes and progress
- **CONTRIBUTING.md** - How to contribute
- **CODE_OF_CONDUCT.md** - Community guidelines

---

## Summary

**5 Native Platforms Supported:**
- iOS, Android, macOS, Windows, Linux

**Build Time:**
- First build: 10-15 minutes (compiles Veilid)
- Subsequent: 2-5 minutes

**One Command to Run:**
```bash
./build_and_bundle.sh && cd mobile && flutter run -d macos
```

**Your app is ready!** 🛤️
