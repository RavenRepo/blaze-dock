# BlazeDock Feature Implementation Status

Last Updated: 2025-12-18

## Overview

BlazeDock is a functional Wayland dock with a solid GTK4 foundation. Some advanced system integration features are in placeholder mode pending Wayland protocol implementation.

## ✅ Complete Features

### Core Foundation
- [x] GTK4 window with layer-shell support
- [x] Floating window fallback for KDE Plasma 6
- [x] TOML configuration system
- [x] Pinned applications
- [x] App launching (async, detached)
- [x] Glassmorphism CSS theme (GTK4-compatible)
- [x] Position configuration (left/right/top/bottom)
- [x] Basic hover effects
- [x] Context menu (right-click)
- [x] Tooltips
- [x] Window dragging (floating mode)

### Core Polish
- [x] Running app indicators (dots) via process scanning
- [x] Process tracker service (efficient single-pass /proc scanning)
- [x] Magnification controller (smooth macOS-style cosine)
- [x] Settings dialog GUI
- [x] Enhanced hover effects (CSS transitions)

### Visual Enhancements
- [x] Badge system (Count, Progress, Attention, Custom)
- [x] Badge CSS styling
- [x] Magnification integration
- [x] Badge-DockItem integration
- [x] Progress rings (Cairo drawing - circular progress indicators)

### System Integration
- [x] Drive monitoring service (lsblk-based)
- [x] Recent files tracking (GIO)
- [x] Theme detection and auto-matching (KDE/GNOME accent colors)

### Intelligence
- [x] Auto-hide logic (opacity-based transitions)
- [x] Edge unhide detection
- [x] Mouse leave/enter tracking

### Keyboard & Shortcuts
- [x] Keyboard navigation (Arrow keys, Enter, Escape)
- [x] Type-to-search overlay (search/filter UI)
- [x] Focus management for dock items

### Multi-Monitor & Polish
- [x] Multi-monitor service (monitor detection, geometry)
- [x] Profile system (create, switch, duplicate profiles)
- [x] Profile presets (work, gaming, presentation)
- [x] Current/primary monitor tracking
- [x] Monitor change notifications

### Deployment
- [x] Installation script (`install.sh`)
- [x] Systemd User Service
- [x] Desktop entry & Autostart
- [x] High-res icons (48px - 256px)
- [x] GitHub CI/CD workflows

---

## ⚠️ Placeholder Features (Pending Implementation)

### Window Tracker
- [ ] Wayland foreign-toplevel protocol integration
- [ ] D-Bus window tracking for KDE (org.kde.KWin)
- [ ] D-Bus window tracking for GNOME (org.gnome.Shell)

**Current Status**: Uses `/proc` scanning for running detection. Window counts and app-to-window mapping not functional.

### D-Bus Integration
- [ ] Unity LauncherEntry badge support (email counts, downloads)
- [ ] FreeDesktop notification listening

**Current Status**: Service initialized but no actual D-Bus listening implemented.

### Window Previews
- [ ] wlr-screencopy-unstable-v1 protocol binding
- [ ] Actual window thumbnail capture

**Current Status**: Shows app icon as fallback. Live window thumbnails not captured.

### Global Shortcuts
- [ ] System-wide keyboard shortcuts (Super+1-9)
- [ ] KDE kglobalaccel integration
- [ ] GNOME GlobalShortcuts portal

**Current Status**: Shortcuts only work when dock window has keyboard focus.

---

## 📊 Feature Summary

| Category | Complete | Placeholder | Total |
|----------|----------|-------------|-------|
| Core Foundation | 11 | 0 | 11 |
| Core Polish | 5 | 0 | 5 |
| Visual Enhancements | 5 | 0 | 5 |
| System Integration | 3 | 2 | 5 |
| Window Previews | 3 | 1 | 4 |
| Intelligence | 3 | 0 | 3 |
| Keyboard & Shortcuts | 3 | 1 | 4 |
| Multi-Monitor | 5 | 0 | 5 |
| Deployment | 5 | 0 | 5 |
| **Total** | **43** | **4** | **47** |

---

## 🔧 Services Architecture

```
BlazeDock Services
├── ProcessTracker      - Running app detection (/proc scanning) ✅
├── WindowTracker       - Window-to-app mapping ⚠️ PLACEHOLDER
├── DBusService         - System event listening ⚠️ PLACEHOLDER  
├── DriveMonitor        - Removable media tracking (lsblk) ✅
├── RecentFilesService  - GIO recent files access ✅
├── RunningAppsService  - Dynamic running apps management ✅
├── ThemeService        - System theme detection (KDE/GNOME) ✅
├── KeyboardService     - In-dock shortcuts ✅ (global ⚠️)
├── MultiMonitorService - Display configuration tracking ✅
└── ScreencopyService   - Window thumbnail capture ⚠️ PLACEHOLDER
```

---

## 🎨 UI Components

All UI components are fully implemented and functional:

```
BlazeDock UI
├── DockWindow          - Main window with layer-shell/floating modes
├── DockItem            - Individual app icons with badges
├── RunningIndicator    - Dots and count badges for running apps
├── MagnificationCtrl   - Cosine-based zoom effect
├── SettingsDialog      - Configuration GUI
├── Badge               - Count/Progress/Attention indicators
├── ProgressRing        - Circular progress (Cairo drawing)
├── WindowPreview       - Hover preview popovers
└── SearchOverlay       - Type-to-search filter UI
```

---

## 🎹 Keyboard Shortcuts

| Shortcut | Action | Status |
|----------|--------|--------|
| Super+1-9 | Launch/focus app at position | ⚠️ Dock focus only |
| Super+D | Toggle dock visibility | ⚠️ Dock focus only |
| Super+/ | Open search overlay | ⚠️ Dock focus only |
| Arrow Keys | Navigate dock items | ✅ Works |
| Enter/Space | Activate focused item | ✅ Works |
| Escape | Close search/popover | ✅ Works |

---

## 🖥️ Runtime Modes

| Compositor | Mode | Layer Shell | Notes |
|------------|------|-------------|-------|
| Sway | Full | ✅ | `BLAZEDOCK_LAYER_SHELL=1` |
| Hyprland | Full | ✅ | `BLAZEDOCK_LAYER_SHELL=1` |
| KDE Plasma 6 | Floating | ❌ | Layer-shell compatibility issues |
| GNOME | Floating | ❌ | Recommended mode |

---

## 🚧 Roadmap: Pending Implementation

### High Priority
1. **Window Tracker** - Wayland foreign-toplevel or D-Bus integration
2. **D-Bus Service** - Unity LauncherEntry badge support

### Medium Priority  
3. **Global Shortcuts** - KDE kglobalaccel / GNOME portal integration
4. **Screencopy** - Live window thumbnail capture

### Low Priority
5. **Testing** - Expand unit and integration test coverage
