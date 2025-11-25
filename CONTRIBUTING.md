# Contributing to BlazeDock

First off, thank you for considering contributing to BlazeDock! 🎉

BlazeDock aims to be the most capable application dock for Linux, and we welcome contributions from the community. This document provides guidelines for contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [How Can I Contribute?](#how-can-i-contribute)
- [Development Setup](#development-setup)
- [Pull Request Process](#pull-request-process)
- [Style Guidelines](#style-guidelines)
- [Community](#community)

## Code of Conduct

This project and everyone participating in it is governed by our [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code. Please report unacceptable behavior to the project maintainers.

## Getting Started

### Prerequisites

- **Fedora 43+** (or compatible Linux distribution)
- **Rust 1.70+** (install via [rustup](https://rustup.rs/))
- **GTK4 development libraries**
- **gtk4-layer-shell development libraries**

### Quick Setup

```bash
# Clone the repository
git clone https://github.com/RavenRepo/blaze-dock.git
cd blaze-dock

# Install dependencies (Fedora)
sudo dnf install -y gcc gtk4-devel gtk4-layer-shell-devel pkg-config

# Build
cargo build

# Run tests
cargo test

# Run the application
cargo run
```

## How Can I Contribute?

### 🐛 Reporting Bugs

Before creating bug reports, please check existing issues to avoid duplicates.

**When reporting a bug, include:**

- Your Fedora/distro version
- Desktop environment (GNOME, KDE Plasma, etc.)
- Wayland compositor version
- Steps to reproduce the issue
- Expected vs actual behavior
- Relevant log output (`RUST_LOG=debug ./blazedock`)
- Screenshots if applicable

**Use this template:**

```markdown
**Environment:**
- OS: Fedora 43
- Desktop: KDE Plasma 6.x on Wayland
- BlazeDock version: 0.1.0

**Description:**
A clear description of the bug.

**Steps to Reproduce:**
1. Start BlazeDock
2. Click on...
3. See error

**Expected Behavior:**
What should happen.

**Actual Behavior:**
What actually happens.

**Logs:**
```
[paste relevant logs here]
```
```

### 💡 Suggesting Features

We love feature suggestions! Please check the [ROADMAP.md](docs/ROADMAP.md) first to see if your idea is already planned.

**For feature requests, include:**

- Clear description of the feature
- Use case / problem it solves
- Proposed implementation approach (if you have ideas)
- Mockups or examples from other applications

### 🔧 Code Contributions

#### Good First Issues

Look for issues labeled `good first issue` - these are great starting points for new contributors.

#### Areas We Need Help

- **Testing on different compositors** (Sway, Hyprland, etc.)
- **Multi-monitor testing**
- **Icon theme compatibility**
- **Accessibility improvements**
- **Documentation and translations**
- **Performance optimization**

## Development Setup

### Project Structure

```
blaze-dock/
├── src/
│   ├── main.rs           # Entry point
│   ├── app.rs            # GTK4 application lifecycle
│   ├── config/           # Configuration management
│   ├── ui/               # User interface components
│   └── utils/            # Utility functions
├── docs/                 # Documentation
├── data/                 # Desktop files, icons
└── scripts/              # Build and install scripts
```

### Building for Development

```bash
# Debug build (faster compilation)
cargo build

# Release build (optimized)
cargo build --release

# With specific features
cargo build --features "dbus,window-tracking"

# Run with debug logging
RUST_LOG=debug cargo run
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

### Code Formatting

We use `rustfmt` for consistent code formatting:

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --check
```

### Linting

We use `clippy` for linting:

```bash
# Run clippy
cargo clippy

# With all targets
cargo clippy --all-targets --all-features
```

## Pull Request Process

### Before Submitting

1. **Fork the repository** and create your branch from `main`
2. **Write tests** for new functionality
3. **Update documentation** if needed
4. **Run the full test suite** and ensure it passes
5. **Run `cargo fmt`** and **`cargo clippy`**
6. **Write a clear commit message**

### Commit Message Format

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
type(scope): description

[optional body]

[optional footer]
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `style`: Code style (formatting, etc.)
- `refactor`: Code change that neither fixes nor adds
- `perf`: Performance improvement
- `test`: Adding tests
- `chore`: Maintenance tasks

**Examples:**
```
feat(ui): add magnification effect on hover
fix(layer-shell): correct anchor positioning on KDE
docs(readme): update installation instructions
perf(icons): implement icon caching system
```

### PR Description Template

```markdown
## Description
Brief description of changes.

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
How was this tested?

## Screenshots (if applicable)

## Checklist
- [ ] Code follows project style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] Tests added/updated
- [ ] All tests pass
```

### Review Process

1. A maintainer will review your PR
2. Address any requested changes
3. Once approved, a maintainer will merge your PR

## Style Guidelines

### Rust Style

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use descriptive variable and function names
- Document public APIs with doc comments
- Prefer `Result` over `panic!` for error handling
- Use `log` crate for logging (not `println!`)

### Documentation Style

- Use clear, concise language
- Include code examples where helpful
- Keep README focused on getting started
- Put detailed docs in `/docs` folder

### CSS Style (GTK4)

- Use CSS custom properties for theming
- Comment sections clearly
- Follow BEM-like naming for classes
- Test with both light and dark themes

## Community

### Getting Help

- **GitHub Discussions**: For questions and general discussion
- **GitHub Issues**: For bugs and feature requests

### Recognition

Contributors are recognized in:
- The project README
- Release notes
- The CONTRIBUTORS file

---

## Thank You! 🙏

Your contributions make BlazeDock better for everyone. We appreciate your time and effort in helping build the best dock for Linux!

