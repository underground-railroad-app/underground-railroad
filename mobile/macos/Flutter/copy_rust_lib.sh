#!/bin/bash
# Automatically copy Rust FFI library into app bundle during build
# This script is called by the Flutter build system

set -e

# Paths relative to mobile/macos/Flutter/
PROJECT_ROOT="../../.."
FFI_LIB="${PROJECT_ROOT}/target/release/libunderground_railroad_ffi.dylib"

# Only proceed if we're in a build (BUILT_PRODUCTS_DIR is set by Xcode)
if [ -z "$BUILT_PRODUCTS_DIR" ]; then
  echo "ℹ️  Not in Xcode build, skipping FFI library copy"
  exit 0
fi

# Build the library if it doesn't exist
if [ ! -f "$FFI_LIB" ]; then
  echo "🦀 Building Rust FFI library..."
  cd "$PROJECT_ROOT"
  cargo build --release -p underground-railroad-ffi
fi

# Copy to Frameworks directory in the app bundle
FRAMEWORKS_DIR="$BUILT_PRODUCTS_DIR/$PRODUCT_NAME.app/Contents/Frameworks"
mkdir -p "$FRAMEWORKS_DIR"

if [ -f "$FFI_LIB" ]; then
  cp -f "$FFI_LIB" "$FRAMEWORKS_DIR/"
  echo "✅ FFI library bundled: $FRAMEWORKS_DIR/libunderground_railroad_ffi.dylib"
  
  # Update install name for proper loading
  install_name_tool -id "@rpath/libunderground_railroad_ffi.dylib" \
    "$FRAMEWORKS_DIR/libunderground_railroad_ffi.dylib" 2>/dev/null || true
fi
