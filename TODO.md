# ✅ Underground Railroad - TODO List

**Last Updated**: October 14, 2025
**Current Status**: 95% Complete

---

## 🔥 IMMEDIATE (Next Session)

### Must Do First (5 minutes)
- [ ] Run `./setup.sh` to generate bridge and models
- [ ] Test app launch: `flutter run -d macos`
- [ ] Verify authentication flow works
- [ ] Test sending a message
- [ ] Check for runtime errors

**Success Criteria**: App runs without crashes, can create PIN, navigate to contacts

---

## 🔥 HIGH PRIORITY (This Week)

### Quick Wins (10-15 hours)

#### QR Code Integration (3 hours)
- [ ] Add `mobile_scanner: ^5.2.3` to pubspec.yaml
- [ ] Add `qr_flutter: ^4.1.0` to pubspec.yaml
- [ ] Create `lib/features/contacts/presentation/qr_scanner_screen.dart`
- [ ] Create `lib/features/contacts/presentation/qr_share_screen.dart`
- [ ] Implement contact data serialization for QR
- [ ] Add signature verification
- [ ] Test contact exchange via QR

#### Biometric Authentication (2 hours)
- [ ] Complete `local_auth` integration in `pin_entry_screen.dart`
- [ ] Add biometric enrollment flow
- [ ] Implement PIN fallback
- [ ] Test on physical devices (Face ID, fingerprint)
- [ ] Handle biometric failures gracefully

#### Settings Screen (5 hours)
- [ ] Create `lib/features/settings/presentation/settings_screen.dart`
- [ ] Add change PIN functionality
- [ ] Add biometric toggle
- [ ] Add auto-lock duration selector
- [ ] Add screenshot protection toggle
- [ ] Add default ephemeral message duration
- [ ] Display Veilid network status
- [ ] Add emergency wipe button
- [ ] Add about/version info

#### Testing & Bug Fixes (10-15 hours)
- [ ] Test on Android (build APK, test on device)
- [ ] Test on iOS (build on simulator, test on device)
- [ ] Test on macOS (current development platform)
- [ ] Test on Linux (build and test)
- [ ] Test on Windows (build and test)
- [ ] Fix runtime bugs
- [ ] Verify encryption works
- [ ] Test duress mode thoroughly
- [ ] Test panic wipe
- [ ] Performance profiling

---

## 🔥🔥 CRITICAL (Weeks 2-3)

### Double Ratchet Implementation (40-60 hours)

#### Research Phase (4-8 hours)
- [ ] Study Signal's Double Ratchet specification
- [ ] Review academic papers
- [ ] Design session state structure
- [ ] Plan key rotation mechanism

#### Rust Implementation (20-30 hours)
- [ ] Create `rust/src/double_ratchet.rs`
- [ ] Implement DH ratchet (Diffie-Hellman key exchange)
- [ ] Implement symmetric key ratchet
- [ ] Implement chain key derivation (KDF chains)
- [ ] Implement message key derivation
- [ ] Add session state management
- [ ] Add key storage and retrieval
- [ ] Write comprehensive tests
- [ ] Document algorithm

#### Flutter Integration (10-15 hours)
- [ ] Create `lib/core/crypto/double_ratchet_service.dart`
- [ ] Create `lib/features/messaging/domain/session_manager.dart`
- [ ] Modify message sending to use DR
- [ ] Modify message receiving to use DR
- [ ] Add session initialization on first message
- [ ] Implement out-of-band key verification
- [ ] Update message models if needed
- [ ] Test forward secrecy

#### Testing (6-8 hours)
- [ ] Unit test DH ratchet
- [ ] Unit test symmetric ratchet
- [ ] Test key rotation
- [ ] Test session persistence
- [ ] Verify forward secrecy property
- [ ] Test out-of-order messages
- [ ] Test session recovery

**Success Criteria**: Messages have perfect forward secrecy - old messages unreadable even if keys compromised

---

## 🟡 MEDIUM PRIORITY (Month 2)

### Real Veilid API (6-10 hours)
- [ ] Replace in-memory VeilidManager with real VeilidAPI
- [ ] Create proper VeilidConfig with paths
- [ ] Configure network settings (ports, protocols)
- [ ] Add bootstrap nodes (production servers)
- [ ] Implement Veilid update handler
- [ ] Test on real Veilid network
- [ ] Platform-specific configuration (iOS, Android, desktop)
- [ ] Handle connection errors
- [ ] Monitor network health

### Alert System (40-60 hours)

#### Models & Database (8 hours)
- [ ] Create `lib/shared/models/alert.dart` with Freezed
- [ ] Update database schema for alerts table
- [ ] Add alert categories enum
- [ ] Add geographic location model (obfuscated)

#### Repository (8 hours)
- [ ] Create `lib/features/alerts/data/alert_repository.dart`
- [ ] Implement create alert
- [ ] Implement broadcast via Veilid DHT
- [ ] Implement receive alerts
- [ ] Add delivery tracking
- [ ] Add expiration handling

#### UI (15 hours)
- [ ] Create `lib/features/alerts/presentation/alerts_screen.dart`
- [ ] Create `lib/features/alerts/presentation/create_alert_screen.dart`
- [ ] Add alert categories (emergency, warning, info, SOS)
- [ ] Add location picker with obfuscation
- [ ] Add alert history view
- [ ] Add alert notifications

#### Veilid Integration (10 hours)
- [ ] Implement broadcast mechanism
- [ ] Add privacy-preserving delivery receipts
- [ ] Add geographic filtering
- [ ] Test broadcast to multiple contacts

### Media Messages (60-80 hours)

#### Image Support (20 hours)
- [ ] Add `image_picker` package
- [ ] Implement image compression
- [ ] Encrypt images with ChaCha20
- [ ] Store in DHT or send directly
- [ ] Create image message widget
- [ ] Add image preview
- [ ] Add image gallery

#### File Support (15 hours)
- [ ] Add `file_picker` package
- [ ] Encrypt files before sending
- [ ] Implement file transfer via Veilid
- [ ] Add file size limits
- [ ] Create file message widget
- [ ] Add file preview

#### Voice Messages (15 hours)
- [ ] Add `record` package
- [ ] Implement voice recording
- [ ] Encrypt audio
- [ ] Add audio playback
- [ ] Create voice message widget
- [ ] Add waveform visualization

#### Video Support (10 hours)
- [ ] Add `video_player` package
- [ ] Encrypt video files
- [ ] Add video compression
- [ ] Create video message widget
- [ ] Add video preview

---

## 🟢 LOW PRIORITY (Future)

### Security Hardening (30-40 hours)
- [ ] Implement anti-debugging measures
- [ ] Add root/jailbreak detection
- [ ] Prevent screenshots in sensitive areas
- [ ] Add code obfuscation configuration
- [ ] Memory encryption for sensitive data
- [ ] Tamper detection
- [ ] Certificate pinning (if using any HTTPS)

### App Disguise Mode (30-40 hours)
- [ ] Create working calculator UI
- [ ] Create working notes UI
- [ ] Implement secret gesture (e.g., enter specific number in calculator)
- [ ] Configure app icon change
- [ ] Configure app name change
- [ ] Hide from recent apps
- [ ] Test effectiveness

### Performance Optimization (20-30 hours)
- [ ] Profile database queries
- [ ] Optimize message list rendering
- [ ] Reduce memory usage
- [ ] Optimize encryption operations
- [ ] Add lazy loading
- [ ] Add pagination
- [ ] Battery optimization

### Advanced Features (Months)
- [ ] Group messaging (40-60 hours)
- [ ] Voice/video calls (160-200 hours)
- [ ] Multi-device sync (120-160 hours)
- [ ] Desktop notifications (10 hours)
- [ ] Message search (20 hours)
- [ ] Message reactions (8 hours)
- [ ] Message replies/threading (15 hours)
- [ ] Contact blocking (5 hours)

---

## 📅 **Suggested Schedule**

### **Week 1** (Oct 15-21, 2025)
**Focus**: Testing & Quick Wins
- Day 1: Setup, testing, bug fixes
- Day 2-3: QR code + biometric + settings
- Day 4-5: Platform testing (all 5 platforms)
- Weekend: Documentation updates

**Deliverable**: Feature-complete MVP

### **Week 2-3** (Oct 22 - Nov 4, 2025)
**Focus**: Double Ratchet (PFS)
- Week 2: Research, design, Rust implementation
- Week 3: Flutter integration, testing
- **Deliverable**: Perfect forward secrecy

### **Week 4** (Nov 5-11, 2025)
**Focus**: Production Veilid
- Real VeilidAPI integration
- Bootstrap node configuration
- Network testing
- **Deliverable**: Real network routing

### **Month 2** (Nov 12 - Dec 9, 2025)
**Focus**: Alert System + Hardening
- Week 5-6: Alert system implementation
- Week 7-8: Security hardening + testing
- **Deliverable**: Alert functionality + hardened security

### **Month 3** (Dec 10 - Jan 6, 2026)
**Focus**: Audit + Polish
- Week 9-10: External security audit
- Week 11-12: Fix audit findings, optimization, final polish
- **Deliverable**: Production v1.0

---

## 🎯 **Milestones**

### **M1: Current State** ✅
- Date: October 14, 2025
- Status: 95% complete
- Features: Auth, messaging, contacts, duress
- Ready for: Testing

### **M2: MVP** (Target: Oct 21, 2025)
- Status: Current + testing + quick wins
- Features: M1 + QR + biometric + settings
- Ready for: Alpha testing with known users

### **M3: Secure v1.0** (Target: Dec 9, 2025)
- Status: MVP + Double Ratchet + audit
- Features: M2 + PFS + real Veilid + hardening
- Ready for: Beta testing / limited release

### **M4: Feature Complete** (Target: Jan 6, 2026)
- Status: v1.0 + all features
- Features: M3 + alerts + media + disguise
- Ready for: Public release

---

## 📋 **Before Considering "Done"**

### **Must Have**
- [x] ~~Core messaging~~ ✅
- [x] ~~E2E encryption~~ ✅
- [x] ~~Duress mode~~ ✅
- [ ] Double Ratchet (PFS) 🔥
- [ ] Security audit passed 🔥
- [ ] Comprehensive tests 🔥
- [ ] All platforms tested

### **Should Have**
- [ ] QR code contact exchange
- [ ] Biometric authentication
- [ ] Settings screen
- [ ] Real Veilid network
- [ ] Alert system
- [ ] Media messages

### **Nice to Have**
- [ ] App disguise mode
- [ ] Group messaging
- [ ] Voice/video calls
- [ ] Multi-device sync

---

## 🎯 **Track Your Progress**

Use this checklist when resuming:
- [ ] Read ROADMAP.md (this file)
- [ ] Read CURRENT_STATE.md
- [ ] Run `./setup.sh`
- [ ] Test app
- [ ] Choose path (MVP/Security/Features)
- [ ] Start with highest priority item
- [ ] Update this TODO.md as you go

---

**All plans consolidated and up to date!** ✅

**Resume point**: Run setup script, test app, then choose your path! 🚀
