# Changelog

All notable changes to BlazeDock will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial project structure with modular architecture
- GTK4 + gtk4-layer-shell integration for Wayland-native dock
- TOML-based configuration system
- Glassmorphism CSS theme (dark, semi-transparent)
- Pinned application support with customizable launchers
- Async application launching (non-blocking UI)
- Desktop entry (.desktop) file parsing
- Basic hover effects and tooltips
- Right-click context menu support
- Configurable dock position (left, right, top, bottom)
- Exclusive zone support (windows don't overlap dock)
- Auto-start desktop entry for login startup
- Comprehensive documentation and roadmap

### Technical
- Rust-based implementation for memory safety and performance
- LTO and release optimizations for small binary size (~1.8MB)
- Modular codebase structure (config, ui, utils modules)
- Cargo feature flags for optional functionality
- CI/CD with GitHub Actions

## [0.1.0] - 2024-XX-XX

### Added
- Initial release
- Core dock functionality
- Basic configuration support

---

## Version History

| Version | Date | Highlights |
|---------|------|------------|
| 0.1.0 | TBD | Initial release |

[Unreleased]: https://github.com/RavenRepo/blaze-dock/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/RavenRepo/blaze-dock/releases/tag/v0.1.0

