#!/bin/sh
set -e

# This script builds the Rust library using cargo directly (with Rust toolchain management inspired by Cargokit)
# It is called from an Xcode build phase

BASEDIR=$(cd "$(dirname "$0")/.."; pwd)

echo "========================================="
echo "Building Rust library for macOS"
echo "========================================="
echo "Base directory: $BASEDIR"
echo "Platform: $PLATFORM_NAME"
echo "Architecture(s): $ARCHS"
echo "Configuration: $CONFIGURATION"
echo "Build products dir: $BUILT_PRODUCTS_DIR"

# Add Rust to PATH if not already there
if [ -d "$HOME/.cargo/bin" ]; then
  export PATH="$HOME/.cargo/bin:$PATH"
fi

# Determine build profile
if [ "$CONFIGURATION" = "Debug" ]; then
  CARGO_PROFILE="debug"
  CARGO_FLAGS=""
else
  CARGO_PROFILE="release"
  CARGO_FLAGS="--release"
fi

echo "Cargo profile: $CARGO_PROFILE"

# Build for each architecture
cd "$BASEDIR/rust"

DYLIB_PATHS=""
for ARCH in $ARCHS; do
  echo "Building for architecture: $ARCH"

  # Map Xcode arch to Rust target
  case "$ARCH" in
    "arm64")
      RUST_TARGET="aarch64-apple-darwin"
      ;;
    "x86_64")
      RUST_TARGET="x86_64-apple-darwin"
      ;;
    *)
      echo "Error: Unsupported architecture $ARCH"
      exit 1
      ;;
  esac

  echo "Rust target: $RUST_TARGET"

  # Ensure target is installed
  rustup target add "$RUST_TARGET" 2>/dev/null || true

  # Build the dynamic library
  echo "Running: cargo build $CARGO_FLAGS --target $RUST_TARGET"
  cargo build $CARGO_FLAGS --target "$RUST_TARGET"

  # Add to the list of dylibs to lipo together
  DYLIB_PATH="$BASEDIR/rust/target/$RUST_TARGET/$CARGO_PROFILE/libunderground_railroad.dylib"
  if [ ! -f "$DYLIB_PATH" ]; then
    echo "Error: Expected dylib not found at $DYLIB_PATH"
    exit 1
  fi
  DYLIB_PATHS="$DYLIB_PATHS $DYLIB_PATH"
done

# Create framework structure
FRAMEWORK_NAME="underground_railroad.framework"
FRAMEWORK_DIR="$BUILT_PRODUCTS_DIR/$FRAMEWORK_NAME"
FRAMEWORK_BINARY="$FRAMEWORK_DIR/Versions/A/underground_railroad"

echo "Creating framework structure at: $FRAMEWORK_DIR"

# Remove old framework if it exists
rm -rf "$FRAMEWORK_DIR"

# Create framework directory structure
mkdir -p "$FRAMEWORK_DIR/Versions/A/Resources"

# Lipo together the libraries for different architectures
echo "Creating universal binary from: $DYLIB_PATHS"
lipo -create $DYLIB_PATHS -output "$FRAMEWORK_BINARY"

# Update the install name of the dylib to be relative to the framework
install_name_tool -id "@rpath/underground_railroad.framework/Versions/A/underground_railroad" "$FRAMEWORK_BINARY"

# Create symlinks for framework structure
cd "$FRAMEWORK_DIR"
ln -sf "Versions/A/underground_railroad" "underground_railroad"
ln -sf "Versions/A/Resources" "Resources"
cd "Versions"
ln -sf "A" "Current"

# Create Info.plist
cat > "$FRAMEWORK_DIR/Versions/A/Resources/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleExecutable</key>
    <string>underground_railroad</string>
    <key>CFBundleIdentifier</key>
    <string>com.example.underground-railroad.rust</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>underground_railroad</string>
    <key>CFBundlePackageType</key>
    <string>FMWK</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>CFBundleSignature</key>
    <string>????</string>
</dict>
</plist>
EOF

# Sign the framework if code signing is enabled
if [ "$CODE_SIGN_IDENTITY" != "" ] && [ "$CODE_SIGN_IDENTITY" != "-" ]; then
  echo "Signing framework..."
  codesign --force --sign "$CODE_SIGN_IDENTITY" "$FRAMEWORK_DIR"
fi

echo "Framework created successfully at: $FRAMEWORK_DIR"
echo "========================================="
