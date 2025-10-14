#!/bin/bash
# Build Rust FFI library for Android
# This creates .so files for Android architectures

set -e

echo "📱 Building FFI for Android..."

# Find Android NDK
if [ -z "$ANDROID_NDK_HOME" ]; then
  if [ -d "$HOME/Library/Android/sdk/ndk" ]; then
    # Find the latest NDK version
    ANDROID_NDK_HOME=$(ls -d "$HOME/Library/Android/sdk/ndk/"* | sort -V | tail -1)
    echo "Found NDK: $ANDROID_NDK_HOME"
  else
    echo "❌ Android NDK not found. Please install it via Android Studio."
    exit 1
  fi
fi

export ANDROID_NDK_HOME

# Set up NDK toolchain paths
NDK_TOOLCHAIN="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin"
export PATH="$NDK_TOOLCHAIN:$PATH"

# Set CC and AR for each target
export CC_aarch64_linux_android="$NDK_TOOLCHAIN/aarch64-linux-android21-clang"
export AR_aarch64_linux_android="$NDK_TOOLCHAIN/llvm-ar"
export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$CC_aarch64_linux_android"

export CC_armv7_linux_androideabi="$NDK_TOOLCHAIN/armv7a-linux-androideabi21-clang"
export AR_armv7_linux_androideabi="$NDK_TOOLCHAIN/llvm-ar"
export CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_LINKER="$CC_armv7_linux_androideabi"

export CC_x86_64_linux_android="$NDK_TOOLCHAIN/x86_64-linux-android21-clang"
export AR_x86_64_linux_android="$NDK_TOOLCHAIN/llvm-ar"
export CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER="$CC_x86_64_linux_android"

# Get project root
PROJECT_ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$PROJECT_ROOT"

# Output directory for Android libraries
ANDROID_LIBS="mobile/android/app/src/main/jniLibs"
mkdir -p "$ANDROID_LIBS"/{arm64-v8a,armeabi-v7a,x86_64}

# Check if rustup/cargo is available
if ! command -v cargo &> /dev/null; then
    echo "❌ cargo not found. Please ensure Rust is installed:"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Check and add targets if needed
echo "Checking Rust targets..."
for target in aarch64-linux-android armv7-linux-androideabi x86_64-linux-android; do
    if ! rustup target list | grep -q "$target (installed)"; then
        echo "Adding target: $target"
        rustup target add $target
    fi
done

echo ""
echo "Building for ARM64 (most modern Android devices)..."
cargo build --target aarch64-linux-android --release -p underground-railroad-ffi 2>&1 | tail -5
if [ -f "target/aarch64-linux-android/release/libunderground_railroad_ffi.so" ]; then
    cp target/aarch64-linux-android/release/libunderground_railroad_ffi.so "$ANDROID_LIBS/arm64-v8a/"
    echo "✅ ARM64 build complete"
else
    echo "⚠️  ARM64 build failed"
fi

echo ""
echo "Building for ARMv7 (older Android devices)..."
cargo build --target armv7-linux-androideabi --release -p underground-railroad-ffi 2>&1 | tail -5
if [ -f "target/armv7-linux-androideabi/release/libunderground_railroad_ffi.so" ]; then
    cp target/armv7-linux-androideabi/release/libunderground_railroad_ffi.so "$ANDROID_LIBS/armeabi-v7a/"
    echo "✅ ARMv7 build complete"
else
    echo "⚠️  ARMv7 build failed"
fi

echo ""
echo "Building for x86_64 (Android emulators)..."
cargo build --target x86_64-linux-android --release -p underground-railroad-ffi 2>&1 | tail -5
if [ -f "target/x86_64-linux-android/release/libunderground_railroad_ffi.so" ]; then
    cp target/x86_64-linux-android/release/libunderground_railroad_ffi.so "$ANDROID_LIBS/x86_64/"
    echo "✅ x86_64 build complete"
else
    echo "⚠️  x86_64 build failed"
fi

echo ""
echo "📦 Checking installed libraries:"
if ls "$ANDROID_LIBS"/*/*.so >/dev/null 2>&1; then
    ls -lh "$ANDROID_LIBS"/*/*.so
    echo ""
    echo "✅ Android FFI libraries ready!"
    echo "   Now run: cd mobile && flutter run -d android"
else
    echo "❌ No libraries were built successfully."
    echo ""
    echo "Troubleshooting:"
    echo "1. Ensure system Rust is installed (not just asdf):"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo "2. Source the cargo environment:"
    echo "   source \$HOME/.cargo/env"
    echo "3. Try again: ./build_android.sh"
    exit 1
fi
