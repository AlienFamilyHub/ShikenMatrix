#!/bin/bash
# Build script to compile Rust library for macOS

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RUST_LIB_DIR="$SCRIPT_DIR/rust-lib"

echo "Building Rust library for macOS..."

# Build for current architecture
cd "$PROJECT_ROOT"
cargo build --lib --release

# Determine target architecture
ARCH=$(uname -m)
TARGET_DIR="target/release"

# Copy the built library
if [ "$ARCH" = "arm64" ]; then
    echo "Copying ARM64 dylib..."
    cp "$PROJECT_ROOT/$TARGET_DIR/libshikenmatrix.dylib" "$RUST_LIB_DIR/"
else
    echo "Copying x86_64 dylib..."
    cp "$PROJECT_ROOT/$TARGET_DIR/libshikenmatrix.dylib" "$RUST_LIB_DIR/"
fi

# Copy the C header
cp "$PROJECT_ROOT/shikenmatrix.h" "$RUST_LIB_DIR/"

echo "✅ Rust library built successfully!"
echo "📁 Library location: $RUST_LIB_DIR/libshikenmatrix.dylib"
echo "📄 Header location: $RUST_LIB_DIR/shikenmatrix.h"
