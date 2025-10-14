#!/bin/bash
# Build Rust FFI library for Windows
# This creates .dll dynamic libraries for Windows

set -e

echo "🪟 Building FFI for Windows..."

# Get project root
PROJECT_ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$PROJECT_ROOT"

# Check if rustup/cargo is available
if ! command -v cargo &> /dev/null; then
    echo "❌ cargo not found. Please ensure Rust is installed:"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Detect platform
if [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]] || [[ "$OSTYPE" == "cygwin" ]]; then
    echo "Building on Windows..."
    TARGET="x86_64-pc-windows-msvc"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    echo "⚠️  Cross-compiling from macOS to Windows..."
    echo "   This requires MinGW toolchain or is best done on Windows."
    echo ""
    echo "   Recommended approaches:"
    echo "   1. Build on Windows machine"
    echo "   2. Use GitHub Actions"
    echo "   3. Use cross-compilation with MinGW"
    echo ""
    read -p "Try with MinGW target anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
    TARGET="x86_64-pc-windows-gnu"
else
    echo "Building on Linux for Windows (cross-compilation)..."
    TARGET="x86_64-pc-windows-gnu"
fi

# Add target if needed
if ! rustup target list | grep -q "$TARGET (installed)"; then
    echo "Adding Windows target..."
    rustup target add $TARGET
fi

# Output directory
WINDOWS_LIBS="mobile/windows/lib"
mkdir -p "$WINDOWS_LIBS"

echo ""
echo "Building for Windows x86_64 ($TARGET)..."
cargo build --target $TARGET --release -p underground-railroad-ffi 2>&1 | tail -10

# Check for both possible output names
DLL_FILE=""
if [ -f "target/$TARGET/release/underground_railroad_ffi.dll" ]; then
    DLL_FILE="target/$TARGET/release/underground_railroad_ffi.dll"
elif [ -f "target/$TARGET/release/libunderground_railroad_ffi.dll" ]; then
    DLL_FILE="target/$TARGET/release/libunderground_railroad_ffi.dll"
fi

if [ -n "$DLL_FILE" ]; then
    cp "$DLL_FILE" "$WINDOWS_LIBS/underground_railroad_ffi.dll"

    echo ""
    echo "✅ Windows FFI library built successfully!"
    echo "   Library: $WINDOWS_LIBS/underground_railroad_ffi.dll"
    ls -lh "$WINDOWS_LIBS"/*.dll
    echo ""
    echo "   Now run: cd mobile && flutter run -d windows"
else
    echo "❌ Windows build failed"
    echo ""
    if [[ "$OSTYPE" == "darwin"* ]]; then
        echo "Cross-compiling to Windows from macOS is complex."
        echo "Recommended: Build on Windows or use GitHub Actions."
        echo ""
        echo "Alternative: Use Docker with Wine"
        echo "  docker run --rm -v \$(pwd):/workspace rust:latest bash -c 'cd /workspace && ./build_windows.sh'"
    fi
    exit 1
fi
