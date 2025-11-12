#!/bin/bash
set -e

echo "Building Keystone for multiple platforms..."

TARGETS=(
    "x86_64-apple-darwin"
    "x86_64-unknown-linux-gnu"
    "x86_64-pc-windows-gnu"
)

mkdir -p dist

for target in "${TARGETS[@]}"; do
    echo "Building for $target..."
    
    if ! rustup target list --installed | grep -q "$target"; then
        echo "Adding target $target..."
        rustup target add "$target"
    fi
    
    cargo build --release --target "$target"
    
    if [ "$target" = "x86_64-pc-windows-gnu" ]; then
        cp "target/$target/release/keystone.exe" "dist/keystone-$target.exe"
    else
        cp "target/$target/release/keystone" "dist/keystone-$target"
    fi
    
    echo "✅ Built for $target"
done

echo ""
echo "Build artifacts:"
ls -lh dist/

echo ""
echo "To build the Docker image:"
echo "  docker build -t keystone:latest ."

