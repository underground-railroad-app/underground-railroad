# Underground Railroad - Quick Start Guide

## 🚀 Get Started in 3 Minutes

### Step 1: Initialize Your Identity

```bash
cargo run --bin urr -- init --name "Alice"
```

You'll be prompted for a password. This creates your encrypted identity.

### Step 2: Check Your Status

```bash
cargo run --bin urr -- status
```

You'll be prompted for your password, then see your network status.

### Step 3: Try a Command

```bash
# Create an emergency request (after entering password)
cargo run --bin urr -- emergency --create

# Register a safe house
cargo run --bin urr -- safe-house --register

# View your contacts
cargo run --bin urr -- contacts
```

---

## 📖 All Commands

### Core Commands

| Command | Description | Example |
|---------|-------------|---------|
| `init` | Create your identity | `cargo run --bin urr -- init --name "Alice"` |
| `status` | Show network status | `cargo run --bin urr -- status` |
| `emergency` | List emergencies | `cargo run --bin urr -- emergency` |
| `emergency --create` | Create emergency request | `cargo run --bin urr -- emergency --create` |
| `safe-house --list` | List available safe houses | `cargo run --bin urr -- safe-house --list` |
| `safe-house --register` | Register your safe house | `cargo run --bin urr -- safe-house --register` |
| `contacts` | List contacts | `cargo run --bin urr -- contacts` |
| `contacts --verbose` | Show full contact details | `cargo run --bin urr -- contacts --verbose` |
| `intel --list` | View intelligence reports | `cargo run --bin urr -- intel --list` |

### Options

| Option | Description | Example |
|--------|-------------|---------|
| `--data-dir <PATH>` | Use custom data directory | `cargo run --bin urr -- --data-dir /secure/path status` |
| `--help` | Show help | `cargo run --bin urr -- --help` |
| `--version` | Show version | `cargo run --bin urr -- --version` |

---

## 🔐 Security

### Your Data is Protected

- **Encryption**: AES-256 (SQLCipher) at rest
- **Password**: Argon2id (memory-hard, 64MB, 3 iterations)
- **Keys**: Ed25519 (signing) + X25519 (encryption)
- **Location**: `~/.underground-railroad/`

### Important Security Notes

⚠️ **NO PASSWORD RECOVERY** - Write down your password securely!

⚠️ **Fingerprint Verification** - Share your 3-word fingerprint with trusted contacts:
```
Your fingerprint: dolphin mountain coffee
```

⚠️ **Data Location** - All encrypted data in `~/.underground-railroad/`:
- `railroad.db` - Encrypted database
- `salt` - Salt for key derivation (not secret, but needed)

---

## 🏃 Quick Examples

### Example 1: Create Emergency Request

```bash
# After initialization
$ cargo run --bin urr -- emergency --create
Password: ********

🆘 Emergency Request
✅ Emergency request created!
   ID: abc-123-def
   Urgency: High

This will broadcast to your trusted network.
```

### Example 2: Register Safe House

```bash
$ cargo run --bin urr -- safe-house --register
Password: ********

🏠 Register Safe House

Safe house name (code name): Green House
Region (approximate, not exact address): Northeast Area
How many people can stay? [2]: 4

✅ Safe house registered!
   ID: safe-456
   Capacity: 4
```

### Example 3: Check Network Status

```bash
$ cargo run --bin urr -- status
Password: ********

🛤️  Underground Railroad - Status

✅ Initialized

Identity: Alice
Fingerprint: dolphin mountain coffee

Network:
  📇 Contacts: 0
  🆘 Active emergencies: 1
  🏠 Available safe houses: 1
  📡 Intelligence reports: 0

🔴 Anonymous network: Not connected
   (Veilid integration pending)
```

---

## 🐛 Troubleshooting

### "Not initialized"
**Solution**: Run `urr init` first

### "Wrong password"
**Solution**: No recovery possible - you must remember your password

### "Foreign key constraint failed"
**Solution**: This has been fixed! Delete `~/.underground-railroad/` and run `init` again

### Build errors
**Solution**: Ensure you have Rust 1.85+:
```bash
rustc --version  # Should be 1.85.0 or higher
```

---

## 🎯 What Works Now

✅ **Local Operation**
- Create encrypted identity
- Store emergencies, safe houses, intelligence
- Manage contacts
- All data encrypted

⏳ **Pending (Veilid Integration)**
- Anonymous networking
- Message broadcasting
- Network coordination
- Offline message delivery

---

## 🛤️ The Underground Railroad

This tool helps coordinate real-world assistance for people fleeing persecution:

- **Emergencies**: Request immediate help
- **Safe Houses**: Find shelter
- **Transportation**: Coordinate movement
- **Intelligence**: Share danger warnings
- **Trust Network**: Verify people safely

**Built with uncompromising security to save lives.**
