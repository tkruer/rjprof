#!/bin/bash

# install.sh - Quick installation script for rjprof
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

print_msg() {
    echo -e "${2}${1}${NC}"
}

print_msg "🔨 Installing rjprof Java Profiler..." $BLUE

# Check requirements
if ! command -v cargo &> /dev/null; then
    print_msg "❌ Rust/Cargo not found. Please install Rust first: https://rustup.rs/" $YELLOW
    exit 1
fi

if [ -z "$JAVA_HOME" ]; then
    print_msg "❌ JAVA_HOME not set. Please set JAVA_HOME to your JDK installation." $YELLOW
    exit 1
fi

print_msg "✅ Requirements check passed" $GREEN

# Build the project
cd "$SCRIPT_DIR"
print_msg "🔨 Building rjprof workspace..." $BLUE
RUSTFLAGS="-Awarnings" cargo build --release --workspace

# Install binaries
print_msg "📦 Installing binaries..." $BLUE
mkdir -p bin
cp target/release/rjprof bin/

# Copy appropriate library
if [ -f "target/release/librjprof.dylib" ]; then
    cp target/release/librjprof.dylib bin/
elif [ -f "target/release/librjprof.so" ]; then
    cp target/release/librjprof.so bin/
elif [ -f "target/release/rjprof.dll" ]; then
    cp target/release/rjprof.dll bin/
fi

# Make scripts executable
chmod +x rjprof.sh

print_msg "✅ Installation complete!" $GREEN
echo
print_msg "Quick start:" $BLUE
echo "  ./rjprof.sh build      # Build everything"
echo "  ./rjprof.sh demo       # Run demo"
echo "  ./rjprof.sh help       # Show help"
echo "  make help              # Show Makefile targets"
echo
print_msg "Binaries installed to: $SCRIPT_DIR/bin/" $GREEN
print_msg "Add to PATH if desired: export PATH=\"$SCRIPT_DIR/bin:\$PATH\"" $YELLOW