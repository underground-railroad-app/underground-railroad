#!/bin/bash
# Copy FFI library into Flutter app bundle
# This runs during the Flutter build process

set -e

# Project paths
REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
FFI_LIB="${REPO_ROOT}/target/release/libunderground_railroad_ffi.dylib"

echo "📦 Copying FFI library into app bundle..."

# Build if needed
if [ ! -f "$FFI_LIB" ]; then
  echo "   Building FFI library first..."
  cd "$REPO_ROOT"
  cargo build --release -p underground-railroad-ffi
fi

# The library will be copied by Xcode build phases
# or we can copy it to the Flutter ephemeral directory
if [ -n "$BUILT_PRODUCTS_DIR" ] && [ -n "$PRODUCT_NAME" ]; then
  FRAMEWORKS_DIR="$BUILT_PRODUCTS_DIR/$PRODUCT_NAME.app/Contents/Frameworks"
  mkdir -p "$FRAMEWORKS_DIR"
  cp -f "$FFI_LIB" "$FRAMEWORKS_DIR/"
  echo "✅ Copied to: $FRAMEWORKS_DIR/"
else
  echo "   (Xcode build variables not set, skipping copy)"
fi
