# Underground Railroad

A secure, anonymous, federated network for coordinating real-world assistance to people fleeing persecution.

## ⚠️ Security Notice

This software is designed to protect people at risk of political persecution. The security model assumes nation-state adversaries with extensive surveillance capabilities.

## Purpose

Like the historical Underground Railroad that helped enslaved people escape to freedom, this modern version helps coordinate:

- 🏠 **Safe Houses**: Secure places to hide and rest
- 🚗 **Transportation**: Movement from danger to safety
- 📦 **Supplies**: Food, medicine, cash, documents
- 🆘 **Emergency Response**: Immediate help when in danger
- 🗺️ **Intelligence**: Real-time information about safe/dangerous areas
- 🤝 **Trust Network**: Verification and vouching for strangers

## Core Principles

1. **Anonymous by Default**: All traffic through Veilid anonymity network
2. **Encrypted Everything**: Hardware-backed encryption at rest, E2E encryption in transit
3. **Zero Metadata**: No logs, no tracking, minimal metadata
4. **Offline Capable**: Bluetooth mesh, SMS fallback, cached data
5. **Dead Simple**: One-tap emergency, chat-app familiar, works under stress
6. **Universal Access**: i18n and a11y as first-class features

## Technology

- **Network**: Veilid (anonymous P2P over Tor/I2P)
- **Encryption**: Post-quantum hybrid (Kyber+X25519, Dilithium+Ed25519)
- **Storage**: SQLCipher with hardware-backed keys
- **Languages**: Rust (core), Flutter (mobile/desktop)
- **Platforms**: iOS, Android, macOS, Windows, Linux (native only - no web)

## Security Model

**Threat Model**: Nation-state adversaries, device seizure, network surveillance, coercion

**Defenses**:
- Hardware security modules (Secure Enclave, StrongBox, TPM)
- Post-quantum cryptography
- Oblivious storage (ORAM patterns)
- Traffic obfuscation (padding, cover traffic, timing randomization)
- Panic wipe and plausible deniability
- Reproducible builds

## Project Status

✅ **Beta** - Core features working, Veilid integration in progress

**Working Features:**
- ✅ Cross-platform (iOS, Android, macOS, Windows, Linux)
- ✅ Encrypted database (SQLCipher/AES-256)
- ✅ Contact management with QR codes
- ⚠️ Emergency coordination (saves locally, broadcasting in development)
- ⚠️ Safe house network (saves locally, DHT announcement in development)
- ✅ Encrypted messaging (hybrid post-quantum X25519+Kyber1024, file-based relay)
- ✅ Data persistence across sessions
- ✅ User ID-based data directories

**Note:** App is fully functional offline. Network broadcasting features are in development.

**Veilid Network Status:**
- ⚠️ Desktop (macOS tested, Windows/Linux untested) - Veilid connects, DHT features in development
- 🔄 Mobile (Android/iOS) - In progress, graceful offline fallback

**Quick Start:**
```bash
# Build and run (macOS)
./build_and_bundle.sh && cd mobile && flutter run -d macos

# Build and run (Android)
./build_android.sh && cd mobile && flutter run -d android

# See BUILD.md for all platforms
```

## License

GPL-3.0-or-later - This software must remain free and open source.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md)

## Security

See [SECURITY.md](SECURITY.md) for threat model and security documentation.

**Found a security issue?** Email underground_railroad_app@proton.me
