#!/bin/bash
# Copy FFI library into macOS app bundle
#
# This script is run as part of the Flutter build process to bundle
# the Underground Railroad FFI library inside the app.

set -e

# Determine the configuration (Debug or Release)
CONFIGURATION="${CONFIGURATION:-Release}"

# Paths
PROJECT_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
FFI_LIB_SOURCE="${PROJECT_ROOT}/target/release/libunderground_railroad_ffi.dylib"
APP_FRAMEWORKS_DIR="${BUILT_PRODUCTS_DIR}/${PRODUCT_NAME}.app/Contents/Frameworks"

echo "🔧 Bundling FFI library for ${CONFIGURATION} configuration..."
echo "   Source: ${FFI_LIB_SOURCE}"
echo "   Destination: ${APP_FRAMEWORKS_DIR}"

# Build the FFI library if it doesn't exist
if [ ! -f "${FFI_LIB_SOURCE}" ]; then
    echo "📦 Building FFI library..."
    cd "${PROJECT_ROOT}"
    cargo build --release --manifest-path "${PROJECT_ROOT}/ffi/Cargo.toml"
fi

# Create Frameworks directory if it doesn't exist
mkdir -p "${APP_FRAMEWORKS_DIR}"

# Copy the library
cp -f "${FFI_LIB_SOURCE}" "${APP_FRAMEWORKS_DIR}/"

# Update the library's install name for proper loading
install_name_tool -id "@executable_path/../Frameworks/libunderground_railroad_ffi.dylib" \
    "${APP_FRAMEWORKS_DIR}/libunderground_railroad_ffi.dylib"

echo "✅ FFI library bundled successfully!"
