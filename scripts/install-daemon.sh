#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "Building daemon..."
cd "$PROJECT_DIR"
cargo build --release -p localdomain-daemon

DAEMON_BIN="$PROJECT_DIR/target/release/localdomain-daemon"
PLIST_SRC="$PROJECT_DIR/resources/com.localdomain.daemon.plist"

echo "Installing daemon (requires admin password)..."
sudo cp "$DAEMON_BIN" /usr/local/bin/localdomain-daemon
sudo chmod 755 /usr/local/bin/localdomain-daemon
sudo cp "$PLIST_SRC" /Library/LaunchDaemons/com.localdomain.daemon.plist
sudo launchctl load /Library/LaunchDaemons/com.localdomain.daemon.plist

echo "Daemon installed and started."
