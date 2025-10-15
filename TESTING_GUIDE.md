# Underground Railroad - Testing Guide

**Purpose**: Verify all security features and messaging functionality work correctly

## 🧪 Pre-Testing Setup

### 1. Generate Code
```bash
# Run setup script (does everything)
./setup.sh

# Or manually:
flutter_rust_bridge_codegen generate
dart run build_runner build --delete-conflicting-outputs
flutter pub get
```

### 2. Verify Build
```bash
# Test Rust crypto
cd rust && cargo test && cd ..

# Check generated files exist
ls lib/generated/bridge.dart
ls lib/shared/models/contact.freezed.dart
ls lib/shared/models/message.freezed.dart
```

### 3. Launch App
```bash
flutter run -d macos
# Or: android, ios, linux, windows
```

---

## 🔐 Security Testing

### Test 1: Initial PIN Setup

**Steps**:
1. Launch app for first time
2. See splash screen → auto-navigate to PIN setup
3. Enter PIN (e.g., "123456")
4. Confirm PIN
5. Enter duress PIN (e.g., "999999")
6. Complete setup

**Expected**:
- ✅ PIN must be 6+ digits
- ✅ Confirmation must match
- ✅ Duress PIN must differ from main PIN
- ✅ Setup completes without errors
- ✅ Navigate to PIN entry screen

**Verify**:
```bash
# Check secure storage (platform-specific)
# iOS: Check Keychain Access app
# Android: Encrypted SharedPreferences created
```

### Test 2: PIN Authentication

**Steps**:
1. Close and relaunch app
2. Enter correct PIN
3. Verify authentication succeeds
4. See contacts screen

**Expected**:
- ✅ Correct PIN authenticates
- ✅ Incorrect PIN shows error
- ✅ 3 failed attempts triggers lockout
- ✅ Database initializes
- ✅ Veilid starts
- ✅ Message listener starts

### Test 3: Duress Mode

**Steps**:
1. Close and relaunch app
2. Enter duress PIN (not main PIN)
3. Verify decoy data appears

**Expected**:
- ✅ Duress PIN authenticates
- ✅ Decoy database loads
- ✅ See fake contacts (Mom, Sarah, Work Group)
- ✅ See fake messages
- ✅ Real contacts hidden
- ✅ No indication you're in duress mode

**Verify**:
```bash
# Check databases exist
ls ~/Library/Application\ Support/com.example.undergroundRailroad/
# Should see: underground_railroad.db and underground_railroad_decoy.db
```

### Test 4: Panic Button

**Steps**:
1. While in real mode (main PIN)
2. Trigger panic wipe (when implemented)
3. Relaunch app
4. Only decoy data remains

**Expected**:
- ✅ Real database deleted
- ✅ Real encryption keys wiped
- ✅ Decoy database intact
- ✅ Can still access decoy with duress PIN
- ✅ No way to recover real data

---

## 💬 Messaging Testing

### Test 5: Add Contact

**Steps**:
1. Tap "+" button on contacts screen
2. Fill in contact details:
   - Name: "Alice"
   - Veilid Route: "VLD1:route:test123..."
   - Public Key: "VLD1:pub:test456..."
3. Tap "Add"

**Expected**:
- ✅ Contact appears in list
- ✅ Safety number generated (6 digits)
- ✅ Shows "Unverified" badge
- ✅ Stored in encrypted database

**Verify**:
```sql
-- Can't read without password (encrypted!)
sqlite3 underground_railroad.db ".schema"
-- Should fail or show gibberish
```

### Test 6: Verify Contact

**Steps**:
1. Open chat with contact
2. Tap shield icon (safety number)
3. Compare number out-of-band
4. Tap "Mark as Verified"

**Expected**:
- ✅ Safety number displays
- ✅ Can copy to clipboard
- ✅ Verification updates contact
- ✅ "Verified" badge appears
- ✅ Green checkmark shows

### Test 7: Send Encrypted Message

**Steps**:
1. Open chat with contact
2. Type message: "Hello from the underground!"
3. Tap send button

**Expected**:
- ✅ Message appears immediately (optimistic UI)
- ✅ Check console: See encryption logs
- ✅ Message stored in database (encrypted)
- ✅ Sent via Veilid route
- ✅ "Sent" checkmark appears

**Verify Encryption**:
```dart
// Check console logs for:
"Encrypting message with ChaCha20-Poly1305"
"Derived shared secret"
"Sending via Veilid route: VLD1:route:..."
"Message stored in encrypted database"
```

### Test 8: Receive Message (Simulated)

**Steps**:
1. Wait for background polling (5-10 seconds)
2. Listener checks for messages
3. If message available, auto-decrypt

**Expected**:
- ✅ Notification appears
- ✅ Message shows in chat
- ✅ Auto-marked as read
- ✅ Decryption successful
- ✅ Signature verified

**Simulate Receiving**:
```dart
// In development, can manually trigger:
final listener = ref.read(messageListenerServiceProvider);
await listener.checkNow();
```

### Test 9: Ephemeral Messages

**Steps**:
1. Tap "..." menu in chat
2. Select "Send Ephemeral Message"
3. Set duration (60 seconds)
4. Send message
5. Wait for expiration

**Expected**:
- ✅ Message sends normally
- ✅ Timer indicator shows
- ✅ After duration, message deletes
- ✅ Deletion is permanent

### Test 10: Message Refresh

**Steps**:
1. In chat screen
2. Tap refresh button
3. Manually trigger sync

**Expected**:
- ✅ Loading indicator shows
- ✅ Messages refresh
- ✅ New messages appear
- ✅ Auto-scroll to latest

---

## 🔒 Encryption Verification

### Test 11: Database Encryption

**Steps**:
```bash
# Navigate to app data directory
cd ~/Library/Application\ Support/com.example.undergroundRailroad/

# Try to open database without password
sqlite3 underground_railroad.db "SELECT * FROM messages;"
```

**Expected**:
- ✅ Error: "file is not a database" OR gibberish
- ✅ Cannot read without encryption key
- ✅ Same for decoy database

### Test 12: Network Encryption

**Steps**:
1. Send message
2. Capture network traffic (Wireshark/tcpdump)
3. Inspect packets

**Expected**:
- ✅ No plaintext in network traffic
- ✅ Veilid onion routing active
- ✅ No metadata (sender/recipient) visible
- ✅ Cannot decrypt without keys

### Test 13: Memory Protection

**Steps**:
1. Send message with sensitive content
2. Check memory dump (if possible)

**Expected**:
- ✅ No plaintext in memory after send
- ✅ Keys zeroed after use (Rust zero-on-drop)
- ✅ No sensitive data in heap

---

## 🎭 Deniability Testing

### Test 14: Dual Database Isolation

**Steps**:
1. In real mode (main PIN):
   - Add contact "Real Contact"
   - Send message "Real message"
2. Close app
3. Reopen with duress PIN
4. Verify real data hidden

**Expected**:
- ✅ Only see decoy contacts
- ✅ Real contacts invisible
- ✅ Real messages invisible
- ✅ Decoy messages show
- ✅ No indication of real data

### Test 15: Panic Wipe

**Steps**:
1. In real mode
2. Trigger panic button
3. Real data wiped
4. Relaunch

**Expected**:
- ✅ Real database deleted
- ✅ Real keys deleted
- ✅ Decoy database intact
- ✅ Can access decoy with duress PIN
- ✅ Cannot recover real data

---

## 🚀 Performance Testing

### Test 16: Message Sending Speed

**Measure**:
- Time from tap send → message appears
- Time to encrypt
- Time to store locally

**Target**:
- < 100ms for UI update
- < 500ms for encryption
- < 1s for total send operation

### Test 17: Message Receiving Speed

**Measure**:
- Polling interval (should be ~5 seconds)
- Time to decrypt received message
- Time to update UI

**Target**:
- 5-10s detection time
- < 500ms for decryption
- < 100ms for UI update

### Test 18: Database Performance

**Test**:
1. Add 100 contacts
2. Send 1000 messages
3. Query messages

**Target**:
- < 50ms for contact list
- < 100ms for message list
- < 10ms for single message

---

## 🐛 Error Testing

### Test 19: Network Errors

**Steps**:
1. Disable network
2. Try sending message

**Expected**:
- ✅ Error message displayed
- ✅ Message saved locally
- ✅ Retry available
- ✅ App doesn't crash

### Test 20: Invalid Data

**Steps**:
1. Add contact with invalid route
2. Try sending message

**Expected**:
- ✅ Validation errors shown
- ✅ No crash
- ✅ Can correct and retry

### Test 21: Database Corruption

**Steps**:
1. Corrupt database file
2. Launch app

**Expected**:
- ✅ Detect corruption
- ✅ Show error message
- ✅ Offer to reset
- ✅ Preserve secure storage

---

## 🔍 Security Audit Checklist

### Encryption
- [ ] All messages encrypted with ChaCha20-Poly1305
- [ ] Unique nonce per message
- [ ] Keys are 32 bytes (256 bits)
- [ ] Proper AEAD usage
- [ ] Signatures verified on receipt

### Key Management
- [ ] Keys stored in platform secure storage
- [ ] Keys derived with Argon2id (65536 iterations)
- [ ] Per-contact key isolation
- [ ] Keys zeroed after use
- [ ] No keys in logs or debug output

### Network Security
- [ ] All traffic via Veilid private routes
- [ ] No plaintext on network
- [ ] No metadata leakage
- [ ] Onion routing active
- [ ] Anonymous sender/receiver

### Storage Security
- [ ] Database encrypted with SQLCipher
- [ ] Dual databases isolated
- [ ] No plaintext in database
- [ ] Emergency wipe works
- [ ] Ephemeral messages actually delete

### Deniability
- [ ] Duress PIN detected correctly
- [ ] Real data hidden in duress mode
- [ ] Decoy data plausible
- [ ] No way to prove real data exists
- [ ] Panic wipe is thorough

---

## 📊 Test Report Template

```markdown
# Test Report - [Date]

## Environment
- Platform: macOS / iOS / Android / etc.
- Flutter: 3.27.x
- Rust: 1.85.0
- Device: [Device name]

## Tests Passed
- [ ] PIN Setup (Test 1)
- [ ] PIN Authentication (Test 2)
- [ ] Duress Mode (Test 3)
- [ ] Panic Button (Test 4)
- [ ] Add Contact (Test 5)
- [ ] Verify Contact (Test 6)
- [ ] Send Message (Test 7)
- [ ] Receive Message (Test 8)
- [ ] Ephemeral Messages (Test 9)
- [ ] Message Refresh (Test 10)
- [ ] Database Encryption (Test 11)
- [ ] Network Encryption (Test 12)
- [ ] Memory Protection (Test 13)
- [ ] Database Isolation (Test 14)
- [ ] Panic Wipe (Test 15)

## Issues Found
[List any bugs, errors, or security concerns]

## Performance Metrics
- Message send time: X ms
- Message receive time: X ms
- Database query time: X ms

## Security Verification
- ✅ No plaintext in database
- ✅ No plaintext on network
- ✅ Keys properly stored
- ✅ Duress mode works
- ✅ Encryption verified

## Recommendations
[Any improvements or fixes needed]
```

---

## 🎯 Next Steps After Testing

### If Tests Pass
1. ✅ Mark messaging as production-ready
2. Add QR code scanning
3. Implement Double Ratchet (PFS)
4. Add media support
5. Security audit by external expert

### If Tests Fail
1. Document failures
2. Fix critical issues
3. Retest
4. Iterate until stable

---

## 🏆 Success Criteria

**Minimum Viable Product**:
- ✅ Authentication works
- ✅ Messages send successfully
- ✅ Messages receive correctly
- ✅ Encryption verified
- ✅ Duress mode functional

**Production Ready**:
- ✅ All tests pass
- ✅ No critical bugs
- ✅ Performance acceptable
- ✅ Security audit passed
- ✅ Documentation complete

---

**Current Status**: Ready for comprehensive testing
**Confidence Level**: High
**Expected Issues**: Minor integration bugs (normal for first test)
**Timeline**: 1-2 days of testing should validate everything

---

**Let's test the most secure messenger built today!** 🔐🧪
