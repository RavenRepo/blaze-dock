#!/bin/bash
# BlazeDock - Dependency Installation Script for Fedora
# Run this script to install all required development dependencies

set -e

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║           BlazeDock Dependency Installer                     ║"
echo "╚══════════════════════════════════════════════════════════════╝"

# Check if running on Fedora
if [ ! -f /etc/fedora-release ]; then
    echo "⚠️  Warning: This script is designed for Fedora."
    echo "   You may need to adapt package names for your distribution."
fi

echo ""
echo "📦 Installing system dependencies..."
sudo dnf install -y \
    gcc \
    gtk4-devel \
    gtk4-layer-shell-devel \
    git \
    pkg-config

echo ""
echo "🦀 Checking for Rust installation..."
if ! command -v rustc &> /dev/null; then
    echo "   Rust not found. Installing via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
else
    echo "   ✓ Rust is already installed: $(rustc --version)"
fi

echo ""
echo "🔧 Verifying installation..."
echo "   Rust: $(rustc --version)"
echo "   Cargo: $(cargo --version)"
echo "   GTK4: $(pkg-config --modversion gtk4 2>/dev/null || echo 'Not found')"
echo "   Layer Shell: $(pkg-config --modversion gtk4-layer-shell 2>/dev/null || echo 'Not found')"

echo ""
echo "✅ All dependencies installed successfully!"
echo ""
echo "Next steps:"
echo "  1. cd $(dirname "$0")/.."
echo "  2. cargo build --release"
echo "  3. ./target/release/blazedock"

