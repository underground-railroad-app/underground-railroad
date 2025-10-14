# Contributing to Underground Railroad

Thank you for your interest in helping build a tool that saves lives!

## Code of Conduct

This project helps people escape persecution. We have zero tolerance for:
- Harassment or discrimination
- Security vulnerabilities introduced intentionally
- Proposals that would harm users or reduce security

## How to Contribute

### Reporting Bugs

1. Check existing issues first
2. Use the bug report template
3. Include logs and steps to reproduce
4. **Security issues**: Email security@underground-railroad.org (do NOT file public issue)

### Suggesting Features

1. Check existing feature requests
2. Use the feature request template
3. Explain how it helps people fleeing persecution
4. Consider security implications

### Contributing Code

1. **Fork** the repository
2. **Create a branch**: `git checkout -b feature/your-feature`
3. **Write tests** for your changes
4. **Follow code style**: `cargo fmt`, `flutter format`
5. **Run tests**: `cargo test`, `flutter test`
6. **Commit**: Use clear commit messages
7. **Push** and create a Pull Request

## Development Setup

### Rust
```bash
rustup install 1.86.0
cargo build
cargo test
```

### Flutter
```bash
flutter pub get
flutter run
flutter test
```

### FFI Bridge
```bash
cd ffi
cargo build --release
```

## Code Standards

### Rust
- **No unsafe code** without thorough review
- **All public APIs documented**
- **Tests for all features**
- **No secrets in code** (use environment variables)
- **Memory safety** (use zeroize for sensitive data)

### Flutter
- **Material Design 3**
- **Accessibility** (screen readers, keyboard nav)
- **i18n ready** (use translation keys)
- **Error handling** (graceful failures)

### Security
- **Encryption by default**
- **Privacy-preserving** (coarse timestamps/locations)
- **Minimal metadata**
- **Security review required** for crypto changes

## Testing

### Unit Tests
```bash
cargo test
flutter test
```

### Integration Tests
```bash
cargo test --test integration
```

### Security Tests
Run before submitting security-critical changes.

## Pull Request Process

1. **Update tests** to cover your changes
2. **Update documentation** if needed
3. **Run formatter**: `cargo fmt`, `flutter format`
4. **Run linter**: `cargo clippy`, `flutter analyze`
5. **Ensure tests pass**: `cargo test --all`
6. **Create PR** with clear description
7. **Wait for review** (may take time for security review)

## Commit Message Format

```
Short summary (50 chars or less)

More detailed explanation if needed. Wrap at 72 characters.
Explain the problem this solves and why this approach was taken.

Fixes #123
```

## Security

### Reporting Security Issues

**DO NOT file public issues for security vulnerabilities!**

Email: underground_railroad_app@proton.me

Include:
- Description of vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

We'll respond within 48 hours.

### Security Review Required For

- Cryptography changes
- Authentication/authorization
- Network protocol changes
- Data storage changes
- Veilid integration changes

## License

By contributing, you agree to license your contribution under GPL-3.0-or-later.

This ensures the Underground Railroad remains free and open source forever.

## Questions?

- **General questions**: Open a discussion
- **Bug reports**: Use issue template
- **Security**: Email security@underground-railroad.org

## Thank You

Every contribution helps people escape persecution. Thank you for making the world safer.

🛤️
