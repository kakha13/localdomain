#!/bin/bash
set -e

# Download Caddy binary for macOS
CADDY_VERSION="2.8.4"
ARCH=$(uname -m)

if [ "$ARCH" = "arm64" ]; then
    PLATFORM="darwin_arm64"
elif [ "$ARCH" = "x86_64" ]; then
    PLATFORM="darwin_amd64"
else
    echo "Unsupported architecture: $ARCH"
    exit 1
fi

DOWNLOAD_URL="https://github.com/caddyserver/caddy/releases/download/v${CADDY_VERSION}/caddy_${CADDY_VERSION}_${PLATFORM}.tar.gz"
TARGET_DIR="$(dirname "$0")/../src-tauri/binaries"

mkdir -p "$TARGET_DIR"

echo "Downloading Caddy v${CADDY_VERSION} for ${PLATFORM}..."
curl -L "$DOWNLOAD_URL" -o /tmp/caddy.tar.gz
tar -xzf /tmp/caddy.tar.gz -C /tmp caddy
mv /tmp/caddy "$TARGET_DIR/caddy"
chmod +x "$TARGET_DIR/caddy"
rm -f /tmp/caddy.tar.gz

echo "Caddy downloaded to $TARGET_DIR/caddy"
