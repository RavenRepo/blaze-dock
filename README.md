<p align="center">
  <img src="docs/logo.png" alt="BlazeDock Logo" width="180" height="180">
</p>

<h1 align="center">🔥 BlazeDock</h1>

<p align="center">
  <strong>A professional, lag-free Wayland dock for Linux</strong>
</p>

<p align="center">
  <a href="https://github.com/RavenRepo/blaze-dock/actions/workflows/ci.yml">
    <img src="https://github.com/RavenRepo/blaze-dock/actions/workflows/ci.yml/badge.svg" alt="CI Status">
  </a>
  <a href="https://github.com/RavenRepo/blaze-dock/releases">
    <img src="https://img.shields.io/github/v/release/RavenRepo/blaze-dock?include_prereleases" alt="Release">
  </a>
  <a href="LICENSE">
    <img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License">
  </a>
  <a href="https://www.rust-lang.org/">
    <img src="https://img.shields.io/badge/rust-1.70%2B-orange.svg" alt="Rust Version">
  </a>
</p>

<p align="center">
  <a href="#features">Features</a> •
  <a href="#installation">Installation</a> •
  <a href="#configuration">Configuration</a> •
  <a href="#roadmap">Roadmap</a> •
  <a href="#contributing">Contributing</a>
</p>

---

## Overview

BlazeDock is a modern, native Wayland dock built with **Rust** and **GTK4**. Designed specifically for Fedora 43+ and other modern Linux distributions, it aims to exceed macOS Dock functionality while maintaining the performance benefits of native code.

### Why BlazeDock?

| Feature | BlazeDock | Plank | Latte Dock |
|---------|-----------|-------|------------|
| Wayland Native | ✅ | ❌ | ⚠️ |
| Memory Safe (Rust) | ✅ | ❌ | ❌ |
| GTK4 | ✅ | GTK3 | Qt |
| Active Development | ✅ | ❌ | ❌ |
| Memory Usage | ~50MB | ~80MB | ~150MB |

## Features

### Current (v0.1.x)
- 🚀 **Native Performance** - Written in Rust for zero-lag operation
- 🎨 **Glassmorphism UI** - Modern dark theme with transparency
- 📌 **Pinned Applications** - Persistent launcher configuration
- 🖥️ **Wayland Native** - Full Layer Shell support
- ⚙️ **TOML Configuration** - Easy customization
- 🔄 **Auto-start** - Optional login startup

### Planned
- 🔍 **Window Previews** - Thumbnail previews on hover
- 📊 **Progress Indicators** - Visual feedback for operations
- 🔔 **Notification Badges** - Real-time notification counts
- 🎯 **Magnification** - macOS-style zoom effect
- ⌨️ **Keyboard Shortcuts** - Super+1-9 to launch apps
- 🖱️ **Drag & Drop** - Reorder and add applications
- 🎨 **Theme Integration** - Auto-match system theme

See the full [Roadmap](docs/ROADMAP.md) for details.

## Installation

### Requirements

- **Fedora 43+** (or compatible distribution)
- **Wayland session** (GNOME, KDE Plasma, Sway, etc.)
- **GTK4** >= 4.10
- **gtk4-layer-shell** >= 1.0

### Quick Install (Fedora)

```bash
# Install dependencies
sudo dnf install -y gcc gtk4-devel gtk4-layer-shell-devel pkg-config git

# Install Rust (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Clone and build
git clone https://github.com/RavenRepo/blaze-dock.git
cd blaze-dock
cargo build --release

# Run
./target/release/blazedock
```

### System-wide Installation

```bash
# Install binary
sudo cp target/release/blazedock /usr/local/bin/

# Enable autostart (optional)
mkdir -p ~/.config/autostart
cp data/blazedock-autostart.desktop ~/.config/autostart/
```

## Configuration

BlazeDock uses a TOML configuration file located at `~/.config/blazedock/blazedock.toml`.

### Example Configuration

```toml
# Dock position: "left", "right", "top", or "bottom"
position = "left"

# Icon size in pixels
icon_size = 48

# Dock width/height
dock_size = 72

# Background opacity (0.0 - 1.0)
opacity = 0.85

# Enable exclusive zone (windows won't overlap)
exclusive_zone = true

# Enable hover zoom effect
hover_zoom = true
hover_zoom_scale = 1.15

# Pinned applications
[[pinned_apps]]
name = "Firefox"
icon = "firefox"
command = "firefox"

[[pinned_apps]]
name = "Terminal"
icon = "org.gnome.Terminal"
command = "gnome-terminal"
```

See [config/blazedock.toml](config/blazedock.toml) for a complete example.

## Usage

### Running

```bash
# Standard launch
blazedock

# With debug logging
RUST_LOG=debug blazedock

# With specific config
BLAZEDOCK_CONFIG=/path/to/config.toml blazedock
```

### Keyboard Shortcuts (Planned)

| Shortcut | Action |
|----------|--------|
| `Super+1-9` | Launch/focus app |
| `Super+Shift+1-9` | Open new window |
| `Super+D` | Toggle dock visibility |

## Screenshots

*Coming soon*

## Roadmap

BlazeDock is under active development. See [docs/ROADMAP.md](docs/ROADMAP.md) for the full development plan.

### Phase Overview

| Phase | Status | Focus |
|-------|--------|-------|
| 1. Core Foundation | ✅ In Progress | Basic dock, config, launching |
| 2. Visual Excellence | 🔜 Next | Animations, theming, badges |
| 3. Window Integration | 📋 Planned | Previews, workspace awareness |
| 4. System Integration | 📋 Planned | D-Bus API, notifications |
| 5. Intelligence | 📋 Planned | Context awareness, learning |
| 6. Power User | 📋 Planned | Scripting, profiles |

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Quick Start for Contributors

```bash
# Fork and clone
git clone https://github.com/YOUR_USERNAME/blaze-dock.git
cd blaze-dock

# Create a branch
git checkout -b feature/my-feature

# Make changes, then
cargo fmt
cargo clippy
cargo test

# Commit and push
git commit -m "feat: add my feature"
git push origin feature/my-feature
```

### Areas We Need Help

- 🧪 Testing on different Wayland compositors
- 🖥️ Multi-monitor testing
- 🎨 Icon theme compatibility
- 📝 Documentation improvements
- 🌍 Translations

## Tech Stack

| Component | Technology |
|-----------|------------|
| Language | Rust 1.70+ |
| UI Framework | GTK4 |
| Wayland Integration | gtk4-layer-shell |
| Configuration | TOML + Serde |
| Async Runtime | Tokio |

## Performance

| Metric | Target | Current |
|--------|--------|---------|
| Startup Time | < 100ms | ~80ms |
| Idle CPU | 0% | 0% |
| Memory Usage | < 50MB | ~25MB |
| Binary Size | < 5MB | 1.8MB |

## License

BlazeDock is licensed under the [MIT License](LICENSE).

## Acknowledgments

- [GTK4](https://gtk.org/) - The UI toolkit
- [gtk4-layer-shell](https://github.com/wmww/gtk4-layer-shell) - Wayland layer shell bindings
- [gtk-rs](https://gtk-rs.org/) - Rust bindings for GTK

## Support

- 🐛 [Report a Bug](https://github.com/RavenRepo/blaze-dock/issues/new?template=bug_report.md)
- 💡 [Request a Feature](https://github.com/RavenRepo/blaze-dock/issues/new?template=feature_request.md)
- 💬 [Discussions](https://github.com/RavenRepo/blaze-dock/discussions)

---

<p align="center">
  Made with ❤️ for the Linux community
</p>
