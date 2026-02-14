#!/bin/bash
set -e

echo "Building LocalDomain .deb package..."

# Build the daemon binary
echo "Building localdomain-daemon..."
cargo build -p localdomain-daemon --release

# Copy daemon binary to resources/ so Tauri bundles it
echo "Copying daemon binary to resources/..."
cp target/release/localdomain-daemon resources/localdomain-daemon

# Build the Tauri .deb package
echo "Building .deb package..."
npx tauri build --bundles deb

# Clean up the daemon binary from resources/ (it's in .gitignore)
echo "Cleaning up..."
rm -f resources/localdomain-daemon

echo "Done! .deb package is in src-tauri/target/release/bundle/deb/"
