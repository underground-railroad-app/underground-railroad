# Usage Guide - Underground Railroad

## Getting Started

The Underground Railroad is a **Flutter mobile/desktop app** for secure assistance coordination.

**There is no CLI tool** - this is an interactive graphical application.

---

## Running the App

### Quick Start

```bash
# macOS
./build_and_bundle.sh
cd mobile && flutter run -d macos

# Android
./build_android.sh
cd mobile && flutter run -d android
```

See [BUILD.md](BUILD.md) for complete build instructions.

---

## First Time Setup

### 1. Create Your Identity

When you first launch the app:
1. Enter your name (can be pseudonym)
2. Create a strong password
3. Your identity is generated with a unique fingerprint

**Important:** Your password encrypts all your data. Choose a strong one and remember it!

### 2. Your Fingerprint

After registration, you'll see three verification words (e.g., "sunset butterfly anchor").

These words are your **fingerprint** - use them to verify your identity with others.

---

## Core Features

### Contact Management

**Add a Contact:**
1. Go to **Contacts** tab
2. Tap "Scan QR Code" button
3. Scan their QR code OR enter fingerprint manually
4. Verify the fingerprint matches what they told you

**Share Your Contact:**
1. Go to **Contacts** tab
2. Tap "My QR Code"
3. Show QR code to others OR
4. Tap "Copy Contact URL" to share via secure channel

**Your contact URL format:**
```
railroad://contact/YourName/word1 word2 word3
```

### Emergency Requests

**Create an Emergency:**
1. Tap the big red **EMERGENCY** button on home screen
2. Select what you need:
   - Safe shelter
   - Transportation
   - Medical assistance
   - Supplies (food, water)
   - Financial help
   - Immediate danger
3. Enter:
   - Region/location (approximate)
   - Urgency level
   - Number of people
4. Tap "Send Emergency Request"

**Current behavior:**
- Saves to your encrypted database
- Desktop: Attempts to broadcast via Veilid (in progress)
- Mobile: Stored locally, can export/share manually

### Safe Houses

**Register a Safe House:**
1. Go to **Safe House** tab
2. Tap "Register Safe House"
3. Enter:
   - Name/description
   - Region
   - Capacity (number of people)
4. Tap "Register"

**Current behavior:**
- Saves to your encrypted database
- Desktop: Would announce via Veilid DHT (not yet implemented)
- Mobile: Stored locally

### Encrypted Messaging

**Send a Message:**
1. Go to **Messages** tab
2. Tap "New Message"
3. Select a contact
4. Type your message
5. Tap send

**How it works:**
- Messages encrypted with hybrid post-quantum encryption (X25519+Kyber1024)
- Currently delivered via local file relay (same device testing)
- **Note:** Cross-device messaging requires both users on same filesystem OR Veilid DHT (in development)

**Receive Messages:**
- Messages appear automatically in conversation
- Pull down to refresh
- Messages persist across sessions

### Intelligence Reports

**Share Information:**
1. Go to **Intel** tab
2. Create reports about:
   - Safe areas
   - Dangerous areas
   - Checkpoint locations
   - Resource availability

**Current status:** UI placeholder, backend not fully implemented

---

## Data & Privacy

### Where Your Data Lives

**macOS/Linux:**
```
~/.underground-railroad/{user-id}/
├── salt                # Password salt (DO NOT DELETE)
├── railroad.db         # Encrypted database
├── veilid/            # Network data
└── messages/          # Encrypted messages
```

**iOS/Android:**
```
{App Documents}/underground-railroad/{user-id}/
└── (same structure)
```

### Data Security

- **All data encrypted** with AES-256 (SQLCipher)
- **Password never stored** - only salt for key derivation
- **User ID derived from password** - same password = same ID
- **Local only** - no cloud storage

### Backing Up Data

**To backup:**
```bash
# macOS/Linux
cp -r ~/.underground-railroad/{your-user-id} /path/to/backup/

# Don't forget the salt!
cp ~/.underground-railroad/salt /path/to/backup/
```

**To restore:**
```bash
# macOS/Linux
cp -r /path/to/backup/{user-id} ~/.underground-railroad/
cp /path/to/backup/salt ~/.underground-railroad/
```

**⚠️ Keep backups encrypted!** The database is encrypted, but protect the salt file.

---

## Network Status

### Desktop (macOS/Windows/Linux)

**Veilid Status:** 🟢 (if connected)

- Connects to Veilid anonymous network
- Uses bootstrap nodes for network access
- Creates private routing contexts
- **Current limitation:** DHT mailboxes not implemented yet

**What this means:**
- Your app connects anonymously
- Network infrastructure is ready
- Message broadcasting via DHT is in development

### Mobile (Android/iOS)

**Veilid Status:** 🔴 (offline)

- Veilid mobile integration in progress
- All features work offline
- Desktop can act as relay nodes

**What this means:**
- App fully functional without network
- Data encrypted and persisted locally
- Can export/import data for sharing

---

## Troubleshooting

### "Database error: file is not a database"

Old corrupted database. Clear and start fresh:

**macOS:**
```bash
rm -rf ~/.underground-railroad
```

**Android:**
```bash
flutter run -d emulator-5554 --uninstall-first
```

### "FFI not loaded"

FFI library not built:
```bash
./build_and_bundle.sh  # macOS
./build_android.sh     # Android
```

### Veilid Shows 🔴

**On mobile:** Expected. Veilid mobile support in development.

**On desktop:** Check logs for specific error. Usually network connectivity issue.

### Lost My Password

**Cannot recover!** Your password encrypts all data. If you lose it, your data is unrecoverable. This is by design for security.

**Prevention:** Write down your password and store securely offline.

---

## Current Limitations

### What Works Offline ✅
- All data storage and encryption
- Contact management
- Creating emergencies/safe houses
- Messaging (on same device or exported/imported)
- QR code exchange

### What Requires Network (In Development) 🔄
- Broadcasting emergencies to trusted contacts
- Receiving help offers
- DHT-based message delivery
- Safe house announcements
- Cross-device real-time messaging

### Platform Differences

| Feature | Desktop | Mobile |
|---------|---------|--------|
| App functionality | ✅ All | ✅ All |
| Data encryption | ✅ Yes | ✅ Yes |
| Veilid connection | ✅ Yes | 🔄 In progress |
| Anonymous networking | 🔄 Partial | ❌ Not yet |

---

## Best Practices

### Security

1. **Use strong passwords** - Your password protects everything
2. **Verify fingerprints in person** - Don't trust QR codes from untrusted sources
3. **Backup your data** - Keep encrypted backups
4. **Don't share your salt** - Keep the salt file private
5. **Use on trusted devices** - Device security matters

### Privacy

1. **Use pseudonyms** - Real names optional
2. **Verify contacts carefully** - Trust network is manual
3. **Test in safe environment first** - Don't rely on untested features in crisis
4. **Understand limitations** - Know what works offline vs online

### Usage Tips

1. **Test emergency features before needing them** - Practice using the app
2. **Exchange contacts in advance** - QR codes work best in person
3. **Keep the app updated** - Security improvements ongoing
4. **Report issues** - See CONTRIBUTING.md

---

## Getting Help

**Documentation:**
- BUILD.md - How to build and run
- PROJECT_STATUS.md - Current feature status
- SECURITY.md - Security model
- DEVELOPMENT.md - Developer guide

**Issues:**
- Check SESSION_PROGRESS.md for known issues
- Report bugs via GitHub issues (when public)

**Security:**
- Email: underground_railroad_app@proton.me

---

## Summary

**Underground Railroad is:**
- ✅ A secure encrypted coordination app
- ✅ Cross-platform (5 native platforms)
- ✅ Offline-first with full functionality
- ✅ Using hybrid post-quantum encryption
- 🔄 Developing anonymous networking features

**Underground Railroad is NOT:**
- ❌ A CLI tool
- ❌ Fully anonymous yet (network features in progress)
- ❌ Cloud-based or requiring servers
- ❌ Production-ready for life-critical situations (still in beta)

**Use responsibly. Test thoroughly. Stay safe.** 🛤️
