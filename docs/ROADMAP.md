# BlazeDock Development Roadmap

## Vision Statement

BlazeDock aims to be the **most capable application dock for Linux**, exceeding macOS Dock functionality while maintaining the performance benefits of Rust and native GTK4 integration.

---

## Phase Overview

| Phase | Name | Duration | Focus |
|-------|------|----------|-------|
| **Phase 1** | Core Foundation | 2-3 weeks | Basic dock functionality, stability |
| **Phase 2** | Visual Excellence | 2-3 weeks | Animations, theming, icons |
| **Phase 3** | Window Integration | 3-4 weeks | Previews, workspace awareness |
| **Phase 4** | Deep System Integration | 4-5 weeks | File ops, notifications, D-Bus |
| **Phase 5** | Intelligence Layer | 3-4 weeks | Context awareness, learning |
| **Phase 6** | Power User Features | 3-4 weeks | Scripting, plugins, profiles |

**Total Estimated Timeline: 17-23 weeks**

---

## Phase 1: Core Foundation ✅ (Current)

### 1.1 Basic Dock Window (DONE)
- [x] GTK4 window with layer-shell integration
- [x] Wayland-native positioning
- [x] Basic CSS theming (glassmorphism)
- [x] TOML configuration system

### 1.2 App Launching (DONE)
- [x] Pinned application support
- [x] Async process spawning
- [x] .desktop file parsing
- [x] Icon theme integration

### 1.3 Core Polish (IN PROGRESS)
- [ ] Fix layer-shell positioning on KDE Plasma
- [ ] Running app indicators (dots)
- [ ] Basic hover effects
- [ ] Tooltips with app names
- [ ] Right-click context menus

### 1.4 Configuration UI
- [ ] Settings dialog (GTK4)
- [ ] Position selector
- [ ] Icon size slider
- [ ] Pinned app management

---

## Phase 2: Visual Excellence

### 2.1 Adaptive Theming
**Goal:** Auto-match GNOME/KDE themes dynamically

```
Implementation:
├── Monitor GSettings for theme changes
├── Parse GTK theme CSS for colors
├── Extract accent colors from system
├── Dynamic CSS variable injection
└── Wallpaper color extraction (optional)
```

**Technical Approach:**
- Use `gio::Settings` to watch `org.gnome.desktop.interface`
- Parse `/usr/share/themes/` CSS files
- Use `gdk_pixbuf` for wallpaper color sampling
- Implement CSS custom properties for theming

**Dependencies:** `gio`, `gdk-pixbuf`, `palette` (color manipulation)

### 2.2 Magnification System
**Goal:** Apple-style cosine magnification with Linux enhancements

```rust
// Cosine-based magnification algorithm
fn calculate_magnification(distance: f64, max_scale: f64, range: f64) -> f64 {
    if distance > range {
        return 1.0;
    }
    let normalized = distance / range;
    let cosine_factor = (1.0 + (std::f64::consts::PI * normalized).cos()) / 2.0;
    1.0 + (max_scale - 1.0) * cosine_factor
}
```

**Features:**
- Smooth 60fps animations via `gtk::TickCallback`
- Multi-level magnification (hover + Ctrl)
- Keyboard navigation magnification
- GPU-accelerated transforms

**Technical Approach:**
- Use CSS `transform: scale()` for GPU acceleration
- Implement custom `EventControllerMotion` for position tracking
- Use `glib::timeout_add` for animation frames
- Calculate neighbor icon scaling in real-time

### 2.3 Icon System Enhancement
**Goal:** Rich icon badges and progress indicators

```
Icon Badge Types:
├── Notification count (number badge)
├── Progress ring (circular progress)
├── State indicator (updating, error)
├── Audio waveform (playing audio)
└── Custom overlays (app-defined)
```

**Technical Approach:**
- Custom `DrawingArea` widget for badge rendering
- Cairo drawing for progress rings
- D-Bus listener for app badge updates
- Icon cache with badge composition

### 2.4 Blur Effects
**Goal:** Real compositor blur (not fake transparency)

**For KDE Plasma:**
- Use KWin blur protocol via layer-shell
- Set `_KDE_NET_WM_BLUR_BEHIND_REGION` property

**For GNOME:**
- Request blur via `gnome-shell` extension (optional)
- Fallback to semi-transparent background

---

## Phase 3: Window Integration

### 3.1 Window Tracking
**Goal:** Know which windows belong to which apps

```
Implementation Stack:
├── Wayland: zwlr_foreign_toplevel_management_v1
├── X11 (fallback): libwnck or _NET_CLIENT_LIST
├── Internal: Window → App mapping
└── Cache: Efficient window state storage
```

**Technical Approach:**
- Use `wayland-client` crate for toplevel protocol
- Implement `ForeignToplevelManager` listener
- Map windows to apps via `app_id` matching
- Track window state (minimized, maximized, focused)

**Dependencies:** `wayland-client`, `wayland-protocols-wlr`

### 3.2 Window Previews
**Goal:** Thumbnail previews on hover/right-click

```
Preview System:
├── Capture: Request window screenshot via protocol
├── Scale: Generate thumbnail at appropriate size
├── Display: Popover with preview grid
├── Interaction: Click to focus, middle-click to close
└── Update: Refresh previews periodically
```

**Technical Approach:**
- Use `wlr-screencopy-unstable-v1` for screenshots
- Generate thumbnails with `gdk-pixbuf`
- Custom `Popover` widget for preview display
- Lazy loading for performance

### 3.3 Workspace Integration
**Goal:** Workspace-aware dock behavior

```
Features:
├── Show workspace indicator per window
├── Filter apps by current workspace
├── Quick workspace switching from dock
├── Pin apps to specific workspaces
└── Workspace preview on modifier+hover
```

**Technical Approach:**
- Monitor `wl_output` for workspace changes
- Use `ext-workspace-unstable-v1` protocol
- KDE: Use KWin's workspace D-Bus API
- GNOME: Use `org.gnome.Shell` D-Bus interface

### 3.4 Window Peeking
**Goal:** Preview window content without switching

```
Peek Behavior:
├── Hover over app icon for 500ms
├── Show live preview overlay
├── Preview follows mouse within icon
├── Click to switch, move away to dismiss
└── Support for grouped windows
```

---

## Phase 4: Deep System Integration

### 4.1 D-Bus API Foundation
**Goal:** Expose BlazeDock to other applications

```
D-Bus Interface: com.blazedock.Dock
├── Methods:
│   ├── AddPinnedApp(desktop_file: string)
│   ├── RemovePinnedApp(app_id: string)
│   ├── SetBadge(app_id: string, badge: variant)
│   ├── SetProgress(app_id: string, progress: double)
│   ├── ShowNotification(app_id: string, message: string)
│   └── ReloadConfig()
├── Signals:
│   ├── AppLaunched(app_id: string)
│   ├── AppClosed(app_id: string)
│   ├── DockClicked(app_id: string, button: int)
│   └── ConfigChanged()
└── Properties:
    ├── PinnedApps: array<string>
    ├── RunningApps: array<string>
    ├── Position: string
    └── Visible: boolean
```

**Technical Approach:**
- Use `zbus` crate for D-Bus integration
- Implement async D-Bus server
- Define XML introspection data
- Document API for developers

**Dependencies:** `zbus`

### 4.2 Notification Integration
**Goal:** Rich notification support beyond badges

```
Notification Features:
├── Real-time badge updates
├── Preview on hover (last N notifications)
├── Progress bars for operations
├── Action buttons in preview
└── Notification grouping by app
```

**Technical Approach:**
- Monitor `org.freedesktop.Notifications` D-Bus
- Implement notification listener service
- Store recent notifications per app
- Custom notification preview widget

### 4.3 File System Integration
**Goal:** Drag-drop and file operations

```
File Features:
├── Drag files to app icons to open
├── Show recent files on right-click
├── Display file operation progress
├── Quick actions (Open With, Copy Path)
└── Folder pinning with contents preview
```

**Technical Approach:**
- Implement `DragDest` on dock items
- Monitor `org.gtk.vfs.Daemon` for operations
- Use `gio::FileMonitor` for recent files
- Custom file action menu

### 4.4 Progress Indicators
**Goal:** Visual feedback for background operations

```
Progress Types:
├── Determinate: Ring fills as progress increases
├── Indeterminate: Spinning animation
├── Multi-stage: Segmented progress
├── Paused: Visual pause indicator
└── Error: Red indicator with details
```

**Technical Approach:**
- Custom Cairo drawing for progress rings
- D-Bus listener for progress updates
- Support Unity LauncherEntry protocol
- Animation system for smooth updates

---

## Phase 5: Intelligence Layer

### 5.1 Context-Aware Auto-Hide
**Goal:** Smart show/hide behavior

```
Intelligence Rules:
├── Hide on fullscreen (configurable)
├── Show on window drag near edge
├── Show on notification arrival
├── Show on keyboard shortcut
├── Per-app exceptions (video players)
└── Time-based (don't hide during presentations)
```

**Technical Approach:**
- Track fullscreen windows via toplevel protocol
- Implement edge detection zones
- Monitor notification D-Bus
- Configurable rule engine

### 5.2 Application State Awareness
**Goal:** Visual indicators for app health

```
State Indicators:
├── Responding/Frozen detection
├── CPU usage level (via /proc)
├── Memory usage indicator
├── Network activity
└── Audio playback state
```

**Technical Approach:**
- Use `procfs` crate for process stats
- PulseAudio/PipeWire D-Bus for audio
- NetworkManager D-Bus for network
- Color-coded subtle indicators

**Dependencies:** `procfs`, `libpulse-binding`

### 5.3 Usage Learning
**Goal:** Adapt to user behavior

```
Learning Features:
├── Track launch frequency per app
├── Time-of-day usage patterns
├── Auto-suggest frequently used apps
├── Smart icon ordering (optional)
└── Project-based app grouping
```

**Technical Approach:**
- SQLite database for usage stats
- Configurable learning algorithms
- Privacy-respecting local-only data
- Export/import usage profiles

**Dependencies:** `rusqlite`

---

## Phase 6: Power User Features

### 6.1 Keyboard-First Design
**Goal:** Full keyboard control

```
Keyboard Shortcuts:
├── Super+1-9: Launch/focus app
├── Super+Shift+1-9: New window
├── Super+D: Toggle dock
├── Super+/: Search dock items
├── Arrow keys: Navigate items
└── Enter: Activate, Menu: Context menu
```

**Technical Approach:**
- Use `gtk4_layer_shell::set_keyboard_mode`
- Global shortcuts via D-Bus (KDE/GNOME)
- Type-to-search filtering
- Focus management system

### 6.2 Advanced Customization
**Goal:** Per-app and profile-based customization

```
Customization:
├── Per-app icon overrides
├── Per-app size overrides
├── Custom icon packs
├── Multiple dock profiles
└── Import/export configurations
```

**Config Structure:**
```toml
[profiles.default]
position = "left"
icon_size = 48

[profiles.presentation]
position = "bottom"
auto_hide = true
icon_size = 64

[app_overrides."firefox"]
icon = "/path/to/custom/firefox.svg"
size = 56

[app_overrides."code"]
icon = "vscode-custom"
actions = ["New Window", "Open Recent"]
```

### 6.3 Scripting & Extensions
**Goal:** User-defined automation

```
Scripting Features:
├── Custom dock items (scripts)
├── Action scripts (on click, on hover)
├── Event hooks (app launch, close)
├── Status scripts (dynamic icons)
└── Plugin API (Lua or WASM)
```

**Technical Approach:**
- Execute scripts via `std::process::Command`
- Script output parsing for status
- Sandboxed plugin runtime (optional)
- Well-documented extension API

### 6.4 Multi-Monitor Support
**Goal:** Excellence on multi-display setups

```
Multi-Monitor Features:
├── Independent dock per monitor
├── Follow-mouse mode
├── Primary-only mode
├── Clone mode (same dock everywhere)
└── Per-monitor position settings
```

**Technical Approach:**
- Monitor `wl_output` for display changes
- Create dock instance per output
- Shared state with independent rendering
- Monitor-aware window tracking

---

## Technical Architecture

### Core Components

```
blazedock/
├── src/
│   ├── main.rs                 # Entry point
│   ├── app.rs                  # GTK4 application lifecycle
│   │
│   ├── config/                 # Configuration management
│   │   ├── mod.rs
│   │   ├── settings.rs         # Main settings
│   │   ├── profiles.rs         # Profile management
│   │   └── schema.rs           # Config validation
│   │
│   ├── ui/                     # User interface
│   │   ├── mod.rs
│   │   ├── window.rs           # Main dock window
│   │   ├── dock_item.rs        # Individual items
│   │   ├── badge.rs            # Badge rendering
│   │   ├── progress.rs         # Progress indicators
│   │   ├── preview.rs          # Window previews
│   │   ├── magnification.rs    # Zoom effects
│   │   ├── animation.rs        # Animation system
│   │   └── style.css           # Theming
│   │
│   ├── services/               # Background services
│   │   ├── mod.rs
│   │   ├── dbus_server.rs      # D-Bus API server
│   │   ├── window_tracker.rs   # Window monitoring
│   │   ├── notification.rs     # Notification listener
│   │   ├── file_ops.rs         # File operation tracking
│   │   └── usage_stats.rs      # Usage learning
│   │
│   ├── wayland/                # Wayland protocols
│   │   ├── mod.rs
│   │   ├── toplevel.rs         # Foreign toplevel
│   │   ├── screencopy.rs       # Screenshot protocol
│   │   └── workspace.rs        # Workspace protocol
│   │
│   └── utils/                  # Utilities
│       ├── mod.rs
│       ├── launcher.rs         # App launching
│       ├── desktop_entry.rs    # .desktop parsing
│       ├── icon_cache.rs       # Icon management
│       └── process.rs          # Process utilities
│
├── data/                       # Resources
│   ├── icons/                  # Bundled icons
│   ├── themes/                 # Built-in themes
│   └── schemas/                # GSettings schemas
│
└── plugins/                    # Extension system
    ├── api/                    # Plugin API
    └── examples/               # Example plugins
```

### Dependency Graph

```
Phase 1 (Core)
    ↓
Phase 2 (Visual) ←──────────────────────┐
    ↓                                   │
Phase 3 (Windows) ───→ Wayland Protocols│
    ↓                                   │
Phase 4 (Integration) ───→ D-Bus API ───┘
    ↓
Phase 5 (Intelligence)
    ↓
Phase 6 (Power User)
```

---

## Performance Targets

| Metric | Target | macOS Dock |
|--------|--------|------------|
| Startup time | < 100ms | ~500ms |
| Idle CPU | 0% | 0.1-0.5% |
| Memory usage | < 50MB | ~100MB |
| Animation FPS | 60fps | 60fps |
| Input latency | < 16ms | ~20ms |
| Icon load time | < 50ms | ~100ms |

---

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| KDE layer-shell issues | High | Medium | Fallback positioning, test on KDE |
| Wayland protocol gaps | Medium | High | X11 fallback where needed |
| Performance regression | Low | High | Continuous profiling, benchmarks |
| Theme compatibility | Medium | Low | Robust CSS parsing, defaults |
| D-Bus complexity | Medium | Medium | Use `zbus` async, good error handling |

---

## Next Steps

### Immediate (This Week)
1. Fix layer-shell positioning on KDE
2. Implement running app indicators
3. Add basic magnification effect
4. Create settings dialog

### Short-term (Next 2 Weeks)
1. Window tracking via foreign-toplevel
2. Basic badge system
3. Keyboard shortcuts (Super+1-9)
4. Theme auto-detection

### Medium-term (Next Month)
1. Window previews
2. D-Bus API foundation
3. Notification integration
4. Advanced animations

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on:
- Code style and conventions
- Testing requirements
- Pull request process
- Feature proposal format

