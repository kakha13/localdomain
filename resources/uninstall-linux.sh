#!/bin/bash
set -e

echo "Uninstalling LocalDomain..."

# Stop and disable systemd service
if systemctl is-active --quiet localdomain-daemon 2>/dev/null; then
    sudo systemctl stop localdomain-daemon
fi
if systemctl is-enabled --quiet localdomain-daemon 2>/dev/null; then
    sudo systemctl disable localdomain-daemon
fi
sudo rm -f /etc/systemd/system/localdomain-daemon.service
sudo systemctl daemon-reload

# Remove daemon binary
sudo rm -f /usr/local/bin/localdomain-daemon

# Remove Caddy binary
sudo rm -f /usr/local/bin/caddy

# Remove CA trust
if [ -f /usr/local/share/ca-certificates/localdomain-ca.crt ]; then
    sudo rm -f /usr/local/share/ca-certificates/localdomain-ca.crt
    sudo update-ca-certificates 2>/dev/null || true
fi

# Clean hosts file
if grep -q "# LocalDomain Start" /etc/hosts; then
    sudo sed -i '/# LocalDomain Start/,/# LocalDomain End/d' /etc/hosts
fi

# Remove socket
sudo rm -f /var/run/localdomain.sock

# Remove data directories
sudo rm -rf /var/lib/localdomain

# Remove app data
rm -rf ~/.local/share/com.localdomain.app

echo "LocalDomain has been uninstalled."
