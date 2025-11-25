#!/bin/bash
# BlazeDock - Installation Script
# Builds and installs BlazeDock to the system

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
INSTALL_DIR="/usr/local/bin"
CONFIG_DIR="$HOME/.config/blazedock"
AUTOSTART_DIR="$HOME/.config/autostart"

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║                BlazeDock Installer                           ║"
echo "╚══════════════════════════════════════════════════════════════╝"

cd "$PROJECT_DIR"

# Build release binary
echo ""
echo "🔨 Building release binary..."
cargo build --release

# Install binary
echo ""
echo "📥 Installing blazedock to $INSTALL_DIR..."
sudo cp target/release/blazedock "$INSTALL_DIR/"
sudo chmod +x "$INSTALL_DIR/blazedock"

# Create config directory
echo ""
echo "📁 Setting up configuration..."
mkdir -p "$CONFIG_DIR"

# Copy default config if not exists
if [ ! -f "$CONFIG_DIR/blazedock.toml" ]; then
    cp config/blazedock.toml "$CONFIG_DIR/"
    echo "   Created default config at $CONFIG_DIR/blazedock.toml"
else
    echo "   Config already exists, skipping..."
fi

# Setup autostart (optional)
echo ""
read -p "🚀 Enable autostart on login? [y/N] " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    mkdir -p "$AUTOSTART_DIR"
    cp data/blazedock-autostart.desktop "$AUTOSTART_DIR/"
    echo "   ✓ Autostart enabled"
fi

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "✅ BlazeDock installed successfully!"
echo ""
echo "To start BlazeDock:"
echo "  blazedock"
echo ""
echo "Configuration file:"
echo "  $CONFIG_DIR/blazedock.toml"
echo ""
echo "To uninstall:"
echo "  sudo rm $INSTALL_DIR/blazedock"
echo "  rm -rf $CONFIG_DIR"
echo "  rm $AUTOSTART_DIR/blazedock-autostart.desktop"
echo "════════════════════════════════════════════════════════════════"

