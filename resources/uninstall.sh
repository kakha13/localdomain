#!/bin/bash
set -e

echo "Uninstalling LocalDomain..."

# Stop and unload daemon
if launchctl list | grep -q com.localdomain.daemon; then
    sudo launchctl unload /Library/LaunchDaemons/com.localdomain.daemon.plist 2>/dev/null || true
fi

# Remove daemon binary and plist
sudo rm -f /Library/LaunchDaemons/com.localdomain.daemon.plist
sudo rm -f /usr/local/bin/localdomain-daemon

# Remove socket
sudo rm -f /var/run/localdomain.sock

# Remove CA from Keychain
if [ -f /var/lib/localdomain/certs/localdomain-ca.crt ]; then
    sudo security remove-trusted-cert -d /var/lib/localdomain/certs/localdomain-ca.crt 2>/dev/null || true
fi

# Clean hosts file
if grep -q "# LocalDomain Start" /etc/hosts; then
    sudo sed -i '' '/# LocalDomain Start/,/# LocalDomain End/d' /etc/hosts
fi

# Remove data directory
sudo rm -rf /var/lib/localdomain
sudo rm -f /var/log/localdomain-daemon.log

# Remove app data
rm -rf ~/Library/Application\ Support/com.localdomain.app

echo "LocalDomain has been uninstalled."
