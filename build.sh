#!/bin/bash
# SysMate Quick Build Script

set -e

echo "🔨 Building SysMate..."
cargo build --release

if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
    echo ""
    echo "To run: cargo run --release --bin sysmate"
    echo "   or:  ./target/release/sysmate"
fi
