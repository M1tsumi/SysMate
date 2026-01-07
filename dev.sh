#!/bin/bash
# SysMate Development Script
# Quick build and run for testing

set -e  # Exit on error

echo "🔍 Checking for required dependencies..."
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found! Please install Rust: https://rustup.rs/"
    exit 1
fi

echo "🔨 Building SysMate (release mode)..."
cargo build --release

if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
    echo ""
    echo "🚀 Launching SysMate..."
    echo "   (Press Ctrl+C to exit)"
    echo ""
    cargo run --release --bin sysmate
else
    echo "❌ Build failed! Check the errors above."
    exit 1
fi
