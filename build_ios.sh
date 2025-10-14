#!/bin/bash
# Build Rust FFI library for iOS
# This creates .a static libraries for iOS architectures

set -e

echo "📱 Building FFI for iOS..."

# Get project root
PROJECT_ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$PROJECT_ROOT"

# Check if rustup/cargo is available
if ! command -v cargo &> /dev/null; then
    echo "❌ cargo not found. Please ensure Rust is installed:"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Check and add targets if needed
echo "Checking Rust targets..."
for target in aarch64-apple-ios x86_64-apple-ios aarch64-apple-ios-sim; do
    if ! rustup target list | grep -q "$target (installed)"; then
        echo "Adding target: $target"
        rustup target add $target
    fi
done

# Output directory for iOS libraries
IOS_LIBS="mobile/ios/Frameworks"
mkdir -p "$IOS_LIBS"

echo ""
echo "Building for iOS ARM64 (iPhone devices)..."
cargo build --target aarch64-apple-ios --release -p underground-railroad-ffi 2>&1 | tail -5
if [ -f "target/aarch64-apple-ios/release/libunderground_railroad_ffi.a" ]; then
    echo "✅ iOS ARM64 build complete"
else
    echo "⚠️  iOS ARM64 build failed"
fi

echo ""
echo "Building for iOS Simulator ARM64 (M1/M2 Macs)..."
cargo build --target aarch64-apple-ios-sim --release -p underground-railroad-ffi 2>&1 | tail -5
if [ -f "target/aarch64-apple-ios-sim/release/libunderground_railroad_ffi.a" ]; then
    echo "✅ iOS Simulator ARM64 build complete"
else
    echo "⚠️  iOS Simulator ARM64 build failed"
fi

echo ""
echo "Building for iOS Simulator x86_64 (Intel Macs)..."
cargo build --target x86_64-apple-ios --release -p underground-railroad-ffi 2>&1 | tail -5
if [ -f "target/x86_64-apple-ios/release/libunderground_railroad_ffi.a" ]; then
    echo "✅ iOS Simulator x86_64 build complete"
else
    echo "⚠️  iOS Simulator x86_64 build failed"
fi

# Create universal binary for simulator
echo ""
echo "Creating universal simulator binary..."
if [ -f "target/aarch64-apple-ios-sim/release/libunderground_railroad_ffi.a" ] && \
   [ -f "target/x86_64-apple-ios/release/libunderground_railroad_ffi.a" ]; then
    lipo -create \
        target/aarch64-apple-ios-sim/release/libunderground_railroad_ffi.a \
        target/x86_64-apple-ios/release/libunderground_railroad_ffi.a \
        -output "$IOS_LIBS/libunderground_railroad_ffi_sim.a"
    echo "✅ Universal simulator library created"
fi

# Copy device library
if [ -f "target/aarch64-apple-ios/release/libunderground_railroad_ffi.a" ]; then
    cp target/aarch64-apple-ios/release/libunderground_railroad_ffi.a "$IOS_LIBS/libunderground_railroad_ffi.a"
fi

echo ""
echo "📦 Checking installed libraries:"
if ls "$IOS_LIBS"/*.a >/dev/null 2>&1; then
    ls -lh "$IOS_LIBS"/*.a
    echo ""
    echo "✅ iOS FFI libraries ready!"
    echo "   Device library: $IOS_LIBS/libunderground_railroad_ffi.a"
    echo "   Simulator library: $IOS_LIBS/libunderground_railroad_ffi_sim.a"
    echo ""
    echo "   Now run: cd mobile && flutter run -d ios"
else
    echo "❌ No libraries were built successfully."
    exit 1
fi
