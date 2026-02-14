#!/bin/bash
set -e

cd "$(dirname "$0")/.."

echo "Building daemon..."
cargo build -p localdomain-daemon

echo "Installing daemon binary (requires sudo)..."
sudo cp target/debug/localdomain-daemon /usr/local/bin/localdomain-daemon

echo "Restarting daemon..."
sudo launchctl kickstart -k system/com.localdomain.daemon

echo "Done. Daemon reloaded."
