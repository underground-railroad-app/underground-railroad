# Security Policy

## Reporting Security Vulnerabilities

**DO NOT file public issues for security vulnerabilities!**

### How to Report

Email: underground_railroad_app@proton.me

GPG Key: (will be published on website)

### What to Include

1. **Description** of the vulnerability
2. **Steps to reproduce** (or proof-of-concept)
3. **Potential impact** (who is affected? how severe?)
4. **Suggested fix** (if you have one)
5. **Your contact info** (for follow-up questions)

### Response Time

- **Acknowledgment**: Within 48 hours
- **Initial assessment**: Within 7 days
- **Fix timeline**: Depends on severity
  - Critical: 24-72 hours
  - High: 7-14 days
  - Medium: 14-30 days
  - Low: Best effort

### Disclosure Policy

We follow **coordinated disclosure**:
1. You report the issue privately
2. We develop and test a fix
3. We release the fix
4. Public disclosure happens together

We'll credit you (if desired) in:
- Security advisory
- CHANGELOG
- Hall of fame (if we create one)

## Security Features

### Current Protections

**Encryption:**
- AES-256 at rest (SQLCipher)
- Hybrid post-quantum: X25519 + Kyber1024 (NIST Level 5)
- ChaCha20-Poly1305 authenticated encryption
- Argon2id password hashing (64MB, 3 iterations, GPU-resistant)
- Ed25519 digital signatures

**Anonymity:**
- Veilid network connection (desktop platforms)
- Private route capability (framework ready)
- 🔄 DHT mailboxes (in development)

**Privacy:**
- Coarse timestamps (5min intervals)
- Coarse locations (regions not addresses)
- Memory zeroization
- Metadata minimization

**Trust:**
- Manual verification required
- Web of trust (no central authority)
- Multiple trust levels

### Known Limitations

**Current:**
- Veilid bootstrapping may reveal network access
- First-time setup requires network connectivity
- DHT records are public (but encrypted)

**In Development:**
- DHT mailbox system for message delivery
- Veilid mobile support (Android/iOS)
- Network broadcasting for emergencies/safe houses

**Planned:**
- Hardware-backed keys (Secure Enclave/TPM)
- Dilithium post-quantum signatures (Kyber1024 already implemented)
- Traffic obfuscation (padding, timing)
- Plausible deniability (hidden volumes)

## Threat Model

### Adversaries We Protect Against

**Nation-state actors with:**
- Network surveillance (global passive adversary)
- Device seizure and forensic analysis
- Traffic analysis capabilities
- Social engineering attempts

### What We Protect

**Your identity:**
- Single identity per password (multiple personas planned)
- No real names required
- Fingerprint verification only

**Your communications:**
- End-to-end encrypted (X25519+Kyber1024+ChaCha20-Poly1305)
- Forward secrecy (ephemeral keys)
- Anonymous routing capability (Veilid - desktop only currently)
- Metadata minimized (coarse timestamps, no precise locations)

**Your data:**
- Encrypted at rest (AES-256)
- Password-protected
- Secure deletion supported

**Your network:**
- Web of trust (decentralized)
- No central authority
- Compartmentalized contacts

### What We DON'T Protect Against

**Physical coercion:**
- Rubber-hose cryptanalysis
- We recommend plausible deniability features (planned)

**Compromised devices:**
- Keyloggers, screen capture
- We recommend clean devices only

**Weak passwords:**
- User education is critical
- We enforce 12+ characters minimum

## Security Best Practices

### For Users

1. **Use strong passwords** (12+ characters, unique)
2. **Verify fingerprints** in person when possible
3. **Trust carefully** (start with in-person verification)
4. **Keep devices secure** (biometric lock, encryption)
5. **Update regularly** (security patches)

### For Developers

1. **No unsafe code** without thorough review
2. **Zeroize sensitive data** in memory
3. **Use constant-time operations** for crypto
4. **Minimize metadata** in all operations
5. **Review dependencies** for vulnerabilities

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | ✅ Yes    |
| < 0.1   | ❌ No     |

## Security Audit Status

- [ ] Independent security audit (planned)
- [ ] Cryptography review (planned)
- [ ] Penetration testing (planned)
- [ ] Bug bounty program (planned)

## Bug Bounty (Future)

We plan to launch a bug bounty program. Details coming soon.

## Acknowledgments

We thank security researchers who responsibly disclose vulnerabilities.

Hall of Fame: (to be created)

---

**Lives depend on this security. Thank you for helping keep people safe.** 🛤️
