#!/bin/bash
set -e

# BlazeDock Installer for Fedora
# Usage: ./install.sh [--user]

INSTALL_PREFIX="/usr/local"
ICON_PREFIX="/usr/share/icons/hicolor"
APPS_PREFIX="/usr/share/applications"
AUTOSTART_PREFIX="/etc/xdg/autostart"

if [[ "$1" == "--user" ]]; then
    INSTALL_PREFIX="$HOME/.local"
    ICON_PREFIX="$HOME/.local/share/icons/hicolor"
    APPS_PREFIX="$HOME/.local/share/applications"
    AUTOSTART_PREFIX="$HOME/.config/autostart"
    mkdir -p "$INSTALL_PREFIX/bin" "$ICON_PREFIX" "$APPS_PREFIX" "$AUTOSTART_PREFIX"
fi

echo "🚀 Building BlazeDock in release mode..."
cargo build --release

echo "📦 Installing BlazeDock binary..."
if [[ "$1" == "--user" ]]; then
    cp target/release/blazedock "$INSTALL_PREFIX/bin/"
else
    sudo cp target/release/blazedock "$INSTALL_PREFIX/bin/"
fi

echo "🖼️  Installing icons..."
# Install various sizes
for size in 48x48 64x64 128x128 256x256; do
    DEST_DIR="$ICON_PREFIX/$size/apps"
    if [[ "$1" == "--user" ]]; then
        mkdir -p "$DEST_DIR"
        cp "data/icons/hicolor/$size/apps/blazedock.png" "$DEST_DIR/"
    else
        sudo mkdir -p "$DEST_DIR"
        sudo cp "data/icons/hicolor/$size/apps/blazedock.png" "$DEST_DIR/"
    fi
done

# Scalable icons
if [[ "$1" == "--user" ]]; then
    mkdir -p "$ICON_PREFIX/scalable/apps"
    cp data/icons/hicolor/scalable/apps/blazedock.svg "$ICON_PREFIX/scalable/apps/"
else
    sudo mkdir -p "$ICON_PREFIX/scalable/apps"
    sudo cp data/icons/hicolor/scalable/apps/blazedock.svg "$ICON_PREFIX/scalable/apps/"
fi

echo "📄 Installing desktop entry..."
if [[ "$1" == "--user" ]]; then
    cp data/blazedock.desktop "$APPS_PREFIX/"
    cp data/blazedock-autostart.desktop "$AUTOSTART_PREFIX/blazedock.desktop"
else
    sudo cp data/blazedock.desktop "$APPS_PREFIX/"
    sudo cp data/blazedock-autostart.desktop "$AUTOSTART_PREFIX/blazedock.desktop"
fi

echo "⚙️  Setting up Systemd User Service..."
SERVICE_DIR="$HOME/.config/systemd/user"
mkdir -p "$SERVICE_DIR"
cat <<EOF > "$SERVICE_DIR/blazedock.service"
[Unit]
Description=BlazeDock - Professional Vertical Dock for Fedora
After=graphical-session.target

[Service]
ExecStart=$INSTALL_PREFIX/bin/blazedock
Restart=always
RestartSec=3

[Install]
WantedBy=graphical-session.target
EOF

systemctl --user daemon-reload
echo "✅ BlazeDock installed successfully!"
echo "To start now: systemctl --user start blazedock"
echo "To enable on boot: systemctl --user enable blazedock"
