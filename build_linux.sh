#!/bin/bash
# Build Rust FFI library for Linux
# This creates .so shared libraries for Linux

set -e

echo "🐧 Building FFI for Linux..."

# Get project root
PROJECT_ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$PROJECT_ROOT"

# Check if rustup/cargo is available
if ! command -v cargo &> /dev/null; then
    echo "❌ cargo not found. Please ensure Rust is installed:"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Detect if we're cross-compiling from macOS
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "⚠️  Cross-compiling from macOS to Linux..."
    echo "   This requires cross-compilation toolchain."
    echo "   For best results, build on Linux directly or use Docker."
    echo ""
    echo "   Recommended: Use GitHub Actions or build on Linux VM"
    echo ""
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi

    # Check for cross-compilation target
    if ! rustup target list | grep -q "x86_64-unknown-linux-gnu (installed)"; then
        echo "Adding Linux target..."
        rustup target add x86_64-unknown-linux-gnu
    fi

    TARGET="x86_64-unknown-linux-gnu"
else
    # Native Linux build
    TARGET="x86_64-unknown-linux-gnu"

    # Add target if needed
    if ! rustup target list | grep -q "$TARGET (installed)"; then
        echo "Adding target: $TARGET"
        rustup target add $TARGET
    fi
fi

# Output directory
LINUX_LIBS="mobile/linux/lib"
mkdir -p "$LINUX_LIBS"

echo ""
echo "Building for Linux x86_64..."
cargo build --target $TARGET --release -p underground-railroad-ffi 2>&1 | tail -10

if [ -f "target/$TARGET/release/libunderground_railroad_ffi.so" ]; then
    cp target/$TARGET/release/libunderground_railroad_ffi.so "$LINUX_LIBS/"
    echo ""
    echo "✅ Linux FFI library built successfully!"
    echo "   Library: $LINUX_LIBS/libunderground_railroad_ffi.so"
    ls -lh "$LINUX_LIBS"/*.so
    echo ""
    echo "   Now run: cd mobile && flutter run -d linux"
else
    echo "❌ Linux build failed"
    echo ""
    echo "If cross-compiling from macOS, you may need:"
    echo "1. Install cross-compilation toolchain"
    echo "2. Or build on actual Linux system"
    echo "3. Or use Docker: docker run --rm -v \$(pwd):/workspace rust:latest bash -c 'cd /workspace && ./build_linux.sh'"
    exit 1
fi
