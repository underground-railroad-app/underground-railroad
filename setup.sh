#!/bin/bash

# Underground Railroad - Setup Script
# Prepares the project for building and running

set -e  # Exit on error

echo "🚂 Underground Railroad - Setup Script"
echo "======================================"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check if running from project root
if [ ! -f "pubspec.yaml" ]; then
    echo -e "${RED}❌ Error: Must run from project root directory${NC}"
    exit 1
fi

echo -e "${BLUE}📋 Step 1: Checking prerequisites...${NC}"

# Check Flutter
if ! command -v flutter &> /dev/null; then
    echo -e "${RED}❌ Flutter not found. Please install Flutter first.${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Flutter found: $(flutter --version | head -n 1)${NC}"

# Check Rust
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Rust/Cargo not found. Please install Rust first.${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Rust found: $(rustc --version)${NC}"

# Check flutter_rust_bridge_codegen
if ! command -v flutter_rust_bridge_codegen &> /dev/null; then
    echo -e "${YELLOW}⚠️  flutter_rust_bridge_codegen not found. Installing...${NC}"
    cargo install flutter_rust_bridge_codegen
    echo -e "${GREEN}✅ flutter_rust_bridge_codegen installed${NC}"
else
    echo -e "${GREEN}✅ flutter_rust_bridge_codegen found${NC}"
fi

echo ""
echo -e "${BLUE}📋 Step 2: Testing Rust crypto...${NC}"
cd rust
if cargo test; then
    echo -e "${GREEN}✅ Rust tests passed${NC}"
else
    echo -e "${RED}❌ Rust tests failed${NC}"
    exit 1
fi
cd ..

echo ""
echo -e "${BLUE}📋 Step 3: Generating Flutter-Rust bridge...${NC}"
# Add cargo bin directory to PATH (works with both regular cargo and asdf)
RUST_VERSION=$(rustc --version | grep -oE '[0-9]+\.[0-9]+\.[0-9]+')
export PATH="$HOME/.local/bin:$HOME/.cargo/bin:$HOME/.asdf/installs/rust/${RUST_VERSION}/bin:$HOME/.asdf/shims:$PATH"
if flutter_rust_bridge_codegen generate; then
    echo -e "${GREEN}✅ Bridge code generated${NC}"
else
    echo -e "${RED}❌ Bridge generation failed${NC}"
    exit 1
fi

echo ""
echo -e "${BLUE}📋 Step 4: Installing Flutter dependencies...${NC}"
if flutter pub get; then
    echo -e "${GREEN}✅ Flutter dependencies installed${NC}"
else
    echo -e "${RED}❌ Flutter pub get failed${NC}"
    exit 1
fi

echo ""
echo -e "${BLUE}📋 Step 5: Generating Dart code (Freezed, Riverpod, etc.)...${NC}"
if dart run build_runner build --delete-conflicting-outputs; then
    echo -e "${GREEN}✅ Dart code generated${NC}"
else
    echo -e "${YELLOW}⚠️  Code generation had warnings (this is often okay)${NC}"
fi

echo ""
echo -e "${BLUE}📋 Step 6: Building Rust library...${NC}"
cd rust
if cargo build; then
    echo -e "${GREEN}✅ Rust library built${NC}"
else
    echo -e "${RED}❌ Rust build failed${NC}"
    exit 1
fi
cd ..

echo ""
echo -e "${GREEN}🎉 Setup complete!${NC}"
echo ""
echo -e "${BLUE}Next steps:${NC}"
echo "  1. Run the app:"
echo "     flutter run -d macos    # or android, ios, linux, windows"
echo ""
echo "  2. Test the messaging system:"
echo "     - Set up a PIN (with optional duress PIN)"
echo "     - Add a test contact"
echo "     - Send encrypted messages"
echo ""
echo -e "${YELLOW}📖 See READY_TO_RUN.md for detailed instructions${NC}"
echo ""
