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

### ✅ Complete Feature Set (v0.1.x)
- 🚀 **Native Performance** - Written in Rust for zero-lag operation (~25MB memory)
- 🎨 **Glassmorphism UI** - Modern dark theme with transparency
- 📌 **Pinned Applications** - Persistent launcher configuration
- 🖥️ **Wayland Native** - Full Layer Shell support (Sway/Hyprland) + floating fallback (KDE/GNOME)
- ⚙️ **TOML Configuration** - Easy customization with live reload
- 🔄 **Auto-start** - Systemd user service and desktop entry
- 🔍 **Window Previews** - Preview UI on hover (shows app icon fallback*)
- 📊 **Progress Rings** - Cairo-drawn circular progress indicators
- 🔔 **Notification Badges** - Count, progress, attention, and custom badges
- 🎯 **Magnification** - macOS-style cosine-based zoom effect
- ⌨️ **Keyboard Shortcuts** - Arrow navigation, type-to-search (Super+1-9 when dock focused*)
- 🖱️ **Window Dragging** - Drag floating dock to reposition
- 🎨 **Theme Integration** - Auto-detect KDE/GNOME accent colors
- 🖥️ **Multi-Monitor** - Primary, All, Follow, and Per-Monitor modes
- 📁 **Profile System** - Multiple dock configurations (work, gaming, presentation)
- 🔄 **Dynamic Running Apps** - macOS-style display of non-pinned running applications
- ⚡ **Auto-Hide** - Intelligent show/hide with edge detection
- 🎯 **Running Indicators** - Dots and window count badges

See the full [Roadmap](docs/ROADMAP.md) and [Feature Status](docs/FEATURE_STATUS.md) for details.

## Installation

### Requirements

- **Fedora 43+** (or compatible distribution)
- **Wayland session** (GNOME, KDE Plasma, Sway, etc.)
- **GTK4** >= 4.10
- **gtk4-layer-shell** >= 1.0

### Installation

```bash
# Clone the repository
git clone https://github.com/RavenRepo/blaze-dock.git
cd blaze-dock

# Run the installer for the current user
./scripts/install.sh --user

# Enable and start the service
systemctl --user enable --now blazedock
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

# Multi-monitor mode: "primary", "all", "follow", "per-monitor"
multi_monitor_mode = "primary"

# Enable keyboard shortcuts (Super+1-9)
enable_shortcuts = true

# Active profile name
active_profile = "default"

# Show running apps dynamically
show_running_apps = true

# Enable window previews on hover
enable_window_previews = true

# Theme mode: "light", "dark", "system"
theme_mode = "system"

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

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Super+1-9` | Launch/focus app at position |
| `Super+D` | Toggle dock visibility |
| `Super+/` | Open search overlay |
| `Arrow Keys` | Navigate dock items |
| `Enter/Space` | Activate focused item |
| `Escape` | Close search/popover |

## Screenshots

*Coming soon*

## Roadmap

BlazeDock has a solid foundation with most UI features complete. Some advanced system integration features are pending Wayland protocol implementation. See [docs/FEATURE_STATUS.md](docs/FEATURE_STATUS.md) for detailed status.

### Phase Overview

| Phase | Status | Focus |
|-------|--------|-------|
| 1. Core Foundation | ✅ Complete | Basic dock, config, launching |
| 2. Visual Excellence | ✅ Complete | Animations, theming, badges, progress rings |
| 3. Window Integration | ⚠️ Partial | UI ready, window tracking placeholder |
| 4. System Integration | ⚠️ Partial | Theme detection works, D-Bus placeholder |
| 5. Intelligence | ✅ Complete | Auto-hide, edge detection, context awareness |
| 6. Power User | ✅ Complete | Profiles, multi-monitor (global shortcuts pending) |

### Known Limitations

> **Note**: These features have UI components but pending backend implementation:

- **Window Tracking**: Uses process scanning; Wayland foreign-toplevel protocol not yet implemented
- **Window Previews**: Shows app icon fallback; live screencopy capture pending
- **Global Shortcuts**: Super+1-9 only works when dock has keyboard focus
- **D-Bus Badges**: Unity LauncherEntry badge support not yet implemented

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
