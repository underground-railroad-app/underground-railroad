# Underground Railroad - Build Guide

## Prerequisites

### Required Tools

1. **Flutter SDK 3.27+**
   ```bash
   flutter --version
   # Should show Flutter 3.27.0 or higher
   ```

2. **Rust 1.75+** (via asdf or rustup)
   ```bash
   # Using asdf (recommended for this project)
   asdf plugin add rust
   asdf install rust 1.85.0
   asdf global rust 1.85.0

   # Or using rustup
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup update
   ```

3. **flutter_rust_bridge_codegen**
   ```bash
   cargo install flutter_rust_bridge_codegen
   ```

### Platform-Specific Requirements

#### macOS
```bash
# Xcode Command Line Tools
xcode-select --install

# CocoaPods
sudo gem install cocoapods
```

#### Linux
```bash
# Install dependencies
sudo apt-get update
sudo apt-get install -y \
  clang cmake ninja-build pkg-config \
  libgtk-3-dev liblzma-dev libstdc++-12-dev
```

#### Windows
```bash
# Install Visual Studio 2022 with C++ development tools
# Download from: https://visualstudio.microsoft.com/downloads/
```

#### Android
```bash
# Install Android Studio
# Download from: https://developer.android.com/studio

# Accept Android licenses
flutter doctor --android-licenses
```

#### iOS
```bash
# Install Xcode from Mac App Store
# Install CocoaPods
sudo gem install cocoapods
```

## Build Steps

### 1. Clone and Setup

```bash
# Navigate to project directory
cd underground-railroad

# Install Rust dependencies and run tests
cd rust
cargo test
cd ..
```

### 2. Generate Flutter-Rust Bridge

```bash
# Generate bridge code
flutter_rust_bridge_codegen generate

# This creates:
# - lib/generated/bridge.dart
# - rust/src/bridge_generated.rs
```

### 3. Install Flutter Dependencies

```bash
flutter pub get

# Generate Dart code (Riverpod, Freezed, etc.)
dart run build_runner build --delete-conflicting-outputs
```

### 4. Build Rust Library

#### Development Build
```bash
cd rust
cargo build
cd ..
```

#### Release Build (Optimized)
```bash
cd rust
cargo build --release
cd ..
```

### 5. Run the Application

#### macOS
```bash
flutter run -d macos
```

#### Linux
```bash
flutter run -d linux
```

#### Windows
```bash
flutter run -d windows
```

#### Android (Device/Emulator)
```bash
# List devices
flutter devices

# Run on connected device
flutter run -d <device-id>

# Or just run (Flutter picks first available)
flutter run
```

#### iOS (Simulator/Device)
```bash
# Open iOS Simulator
open -a Simulator

# Run on simulator
flutter run -d "iPhone 15 Pro"

# For physical device, you'll need Apple Developer account
flutter run -d <device-id>
```

## Platform-Specific Build Instructions

### Android APK/Bundle

```bash
# Debug APK
flutter build apk --debug

# Release APK
flutter build apk --release

# App Bundle (for Play Store)
flutter build appbundle --release
```

Output: `build/app/outputs/flutter-apk/app-release.apk`

### iOS IPA

```bash
# Build for iOS
flutter build ios --release

# Create IPA (requires Xcode)
# 1. Open ios/Runner.xcworkspace in Xcode
# 2. Product > Archive
# 3. Distribute App
```

### macOS App

```bash
# Build macOS app
flutter build macos --release
```

Output: `build/macos/Build/Products/Release/underground_railroad.app`

### Linux Binary

```bash
# Build Linux app
flutter build linux --release
```

Output: `build/linux/x64/release/bundle/underground_railroad`

### Windows Executable

```bash
# Build Windows app
flutter build windows --release
```

Output: `build/windows/x64/runner/Release/underground_railroad.exe`

## Troubleshooting

### Bridge Generation Issues

If bridge generation fails:
```bash
# Clean and regenerate
rm -rf lib/generated rust/src/bridge_generated.rs
flutter_rust_bridge_codegen generate
```

### Build Runner Issues

```bash
# Clean build cache
flutter clean
dart run build_runner clean

# Rebuild
flutter pub get
dart run build_runner build --delete-conflicting-outputs
```

### Rust Compilation Issues

```bash
# Update Rust toolchain
rustup update

# Clean Rust build
cd rust
cargo clean
cargo build
```

### Platform-Specific Issues

#### macOS: "Developer cannot be verified"
```bash
# Remove quarantine attribute
xattr -cr build/macos/Build/Products/Release/underground_railroad.app
```

#### Linux: Missing libraries
```bash
sudo apt-get install -y libsqlcipher-dev libsecret-1-dev
```

#### Android: NDK not found
```bash
# Install NDK via Android Studio
# Tools > SDK Manager > SDK Tools > NDK (Side by side)
```

## Development Workflow

### Hot Reload (During Development)

```bash
# Run in debug mode
flutter run -d macos

# After code changes:
# Press 'r' for hot reload
# Press 'R' for hot restart
# Press 'q' to quit
```

### Running Tests

```bash
# Rust tests
cd rust && cargo test && cd ..

# Flutter tests
flutter test

# Integration tests (once implemented)
flutter test integration_test/
```

### Code Generation (After Model Changes)

```bash
# Watch mode (auto-regenerates on save)
dart run build_runner watch --delete-conflicting-outputs

# One-time generation
dart run build_runner build --delete-conflicting-outputs
```

## Production Builds

### Android (Play Store)

```bash
# Generate signing key (first time only)
keytool -genkey -v -keystore ~/underground-railroad-key.jks \
  -keyalg RSA -keysize 2048 -validity 10000 \
  -alias underground-railroad

# Build signed bundle
flutter build appbundle --release
```

### iOS (App Store)

```bash
# Build for release
flutter build ios --release

# Archive and distribute via Xcode
# Product > Archive > Distribute App
```

### Desktop Installers

#### macOS DMG
```bash
# Build app
flutter build macos --release

# Create DMG (requires create-dmg)
brew install create-dmg
create-dmg build/macos/Build/Products/Release/underground_railroad.app
```

#### Windows Installer (MSIX)
```bash
# Build Windows app
flutter build windows --release

# Package as MSIX (requires MSIX packaging tool)
flutter pub run msix:create
```

#### Linux Package (.deb)
```bash
# Build Linux app
flutter build linux --release

# Create .deb package (manual process or use fpm)
```

## Security Considerations for Production

1. **Enable Code Obfuscation**
   ```bash
   flutter build apk --release --obfuscate --split-debug-info=symbols/
   ```

2. **Remove Debug Symbols**
   - Already configured in `rust/Cargo.toml` with `strip = true`

3. **Verify No Debug Code**
   ```bash
   # Search for debug prints
   grep -r "print(" lib/
   grep -r "println!" rust/src/
   ```

4. **Use Release Mode**
   - Always build with `--release` flag for production

5. **Secure Key Storage**
   - Never commit signing keys to git
   - Store keys in secure location
   - Use CI/CD secrets for automated builds

## Next Steps

After successful build:
1. Test PIN setup and authentication
2. Test duress mode (once decoy data is generated)
3. Implement messaging features
4. Complete Veilid integration
5. Add biometric authentication
6. Implement Double Ratchet for PFS

## Support

For build issues:
- Check Flutter doctor: `flutter doctor -v`
- Verify Rust installation: `cargo --version`
- Clean everything: `flutter clean && cd rust && cargo clean`

---

Built with security in mind. Every build is optimized for privacy and anonymity.
