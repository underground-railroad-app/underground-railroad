# Underground Railroad - Mobile App

Flutter mobile application for iOS and Android.

## Features

### Implemented UI
- ✅ Onboarding screen (name + password)
- ✅ Home dashboard with emergency button
- ✅ Emergency request screen
- ✅ Safe house management
- ✅ Contacts screen with QR code
- ✅ Intelligence reports screen
- ✅ Bottom navigation

### Pending Integration
- ⏳ FFI bindings to Rust core
- ⏳ Actual QR code generation/scanning
- ⏳ Database operations via FFI
- ⏳ Biometric unlock
- ⏳ Push notifications

## Setup

### Prerequisites
```bash
# Install Flutter
# https://flutter.dev/docs/get-started/install

# Verify installation
flutter doctor
```

### Run on iOS Simulator
```bash
cd mobile
flutter run -d ios
```

### Run on Android Emulator
```bash
cd mobile
flutter run -d android
```

### Build Release
```bash
# Android APK
flutter build apk --release

# iOS (requires Mac + Xcode)
flutter build ios --release
```

## Architecture

```
mobile/
├── lib/
│   ├── main.dart                 # App entry point
│   ├── state/
│   │   └── app_state.dart        # Global state management
│   ├── screens/
│   │   ├── onboarding_screen.dart    # First-time setup
│   │   ├── home_screen.dart          # Dashboard
│   │   ├── emergency_screen.dart     # Emergency request
│   │   ├── safe_house_screen.dart    # Safe houses
│   │   ├── contacts_screen.dart      # Trust network
│   │   └── intel_screen.dart         # Intelligence
│   └── ffi/
│       └── (TODO) Rust FFI bindings
└── pubspec.yaml                  # Dependencies
```

## Next Steps

1. **FFI Bridge** - Connect Flutter to Rust core
2. **QR Codes** - Generate/scan contact QR codes
3. **Biometric** - Fingerprint/Face ID unlock
4. **Push Notifications** - Emergency alerts
5. **i18n** - Multi-language support

## UI Preview

### Onboarding
```
┌─────────────────────────┐
│    🚂 (train icon)      │
│  Underground Railroad   │
│                         │
│  [Your Name          ]  │
│  [Master Password    ]  │
│  [Confirm Password   ]  │
│                         │
│  ℹ️ Security Notice      │
│  Everything encrypted   │
│  We can't recover pwd   │
│                         │
│  [  Get Started  ]      │
└─────────────────────────┘
```

### Home Screen
```
┌─────────────────────────┐
│ Underground Railroad  ⚙️│
├─────────────────────────┤
│                         │
│ 👤 Alice                │
│    dolphin mountain...  │
│                         │
│ ┌─────────────────────┐ │
│ │    🆘 EMERGENCY     │ │
│ │   I need help now   │ │
│ └─────────────────────┘ │
│                         │
│ Network Status:         │
│  👥 Contacts: 5         │
│  🆘 Emergencies: 2      │
│  🏠 Safe Houses: 3      │
│                         │
│ 🔴 Not connected        │
│                         │
├─────────────────────────┤
│ 🏠 ⛑️ 👥 ℹ️  (nav bar)   │
└─────────────────────────┘
```

## Distribution

### Google Play Store
- APK size: ~20MB
- Minimum Android: 5.0 (API 21)
- Permissions: Camera (QR), Storage, Network

### Apple App Store
- IPA size: ~30MB
- Minimum iOS: 12.0
- Permissions: Camera, Face ID, Notifications

### F-Droid (Open Source Store)
- For users in countries with censored app stores
- Fully open source, reproducible builds

### Direct APK
- For sideloading in restricted regions
- Distributed via secure channels
- Bluetooth sharing between devices
