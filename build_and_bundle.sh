#!/bin/bash
# Build and bundle the Underground Railroad app with FFI library
# This script ensures the Rust FFI library is built and properly bundled

set -e

echo "🚂 Underground Railroad - Build & Bundle Script"
echo "================================================"

# Get the project root
PROJECT_ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$PROJECT_ROOT"

# Detect target platform from command line argument
TARGET_PLATFORM="${1:-macos}"

echo ""
echo "🎯 Target platform: $TARGET_PLATFORM"

# Step 1: Build the Rust FFI library
echo ""
echo "📦 Step 1: Building Rust FFI library for $TARGET_PLATFORM..."

case "$TARGET_PLATFORM" in
    android)
        echo "Building for Android architectures..."
        ./build_android.sh
        ;;
    ios)
        echo "Building for iOS architectures..."
        ./build_ios.sh
        ;;
    linux)
        echo "Building for Linux..."
        ./build_linux.sh
        ;;
    windows)
        echo "Building for Windows..."
        ./build_windows.sh
        ;;
    macos|*)
        # Build for macOS/desktop
        cargo build --release -p underground-railroad-ffi
        echo "✅ FFI library built successfully"
        ;;
esac

# Step 2: Copy library to mobile directories for easy access
echo ""
echo "📋 Step 2: Copying FFI library..."
# Copy to macos directory
cp -v target/release/libunderground_railroad_ffi.dylib mobile/macos/
# Copy to build Frameworks if it exists (for running app)
if [ -d "mobile/build/macos/Build/Products/Debug/underground_railroad.app/Contents/Frameworks" ]; then
  cp -v target/release/libunderground_railroad_ffi.dylib mobile/build/macos/Build/Products/Debug/underground_railroad.app/Contents/Frameworks/
  echo "✅ Library copied to app bundle"
fi
echo "✅ Library copied"

# Step 3: Build/run Flutter app
echo ""
echo "🎨 Step 3: Ready to build Flutter app"
echo ""
echo "To run the app now, use one of these commands:"
echo "  cd mobile && flutter run -d macos"
echo "  cd mobile && flutter run -d ios"
echo "  cd mobile && flutter run -d android"
echo "  cd mobile && flutter run -d linux"
echo "  cd mobile && flutter run -d windows"
echo ""
echo "To build for distribution:"
echo "  cd mobile && flutter build macos --release"
echo "  cd mobile && flutter build ios --release"
echo "  cd mobile && flutter build apk --release"
echo "  cd mobile && flutter build linux --release"
echo "  cd mobile && flutter build windows --release"
echo ""
echo "Note: Native platforms only - web excluded for security"
echo ""
echo "✅ Build preparation complete!"
