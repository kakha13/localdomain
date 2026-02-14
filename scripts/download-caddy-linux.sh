#!/bin/bash
set -e

# Download Caddy binary for Linux
CADDY_VERSION="2.8.4"
ARCH=$(uname -m)

if [ "$ARCH" = "aarch64" ]; then
    PLATFORM="linux_arm64"
elif [ "$ARCH" = "x86_64" ]; then
    PLATFORM="linux_amd64"
else
    echo "Unsupported architecture: $ARCH"
    exit 1
fi

DOWNLOAD_URL="https://github.com/caddyserver/caddy/releases/download/v${CADDY_VERSION}/caddy_${CADDY_VERSION}_${PLATFORM}.tar.gz"

echo "Downloading Caddy v${CADDY_VERSION} for ${PLATFORM}..."
curl -L "$DOWNLOAD_URL" -o /tmp/caddy.tar.gz
tar -xzf /tmp/caddy.tar.gz -C /tmp caddy
sudo mv /tmp/caddy /usr/local/bin/caddy
sudo chmod +x /usr/local/bin/caddy
rm -f /tmp/caddy.tar.gz

echo "Caddy installed to /usr/local/bin/caddy"
