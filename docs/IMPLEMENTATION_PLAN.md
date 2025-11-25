# BlazeDock Implementation Plan

## Immediate Priority: Fix Core Issues

### Issue 1: Layer Shell Positioning on KDE
**Status:** In Progress

**Root Cause Analysis:**
- KDE Plasma's layer-shell implementation may have timing requirements
- Need to ensure `init_layer_shell()` is called before window realization

**Solution Approach:**
1. Add explicit layer-shell support check
2. Try alternative initialization order
3. Add fallback for non-layer-shell mode
4. Test with different KDE versions

---

## Sprint 1: Core Polish (Week 1-2)

### Task 1.1: Running App Indicators
**Priority:** High | **Effort:** 4 hours

```rust
// Add to DockItem
struct RunningIndicator {
    dot_count: u8,      // Number of dots (windows)
    is_focused: bool,   // Current focused app
    is_urgent: bool,    // Needs attention
}
```

**Implementation:**
- Add indicator widget below icons
- Draw dots using Cairo
- Update via window tracking (future) or process check (now)

### Task 1.2: Basic Magnification
**Priority:** High | **Effort:** 8 hours

```rust
// Magnification calculator
pub struct MagnificationController {
    max_scale: f64,        // e.g., 1.5 = 150%
    range_items: usize,    // How many neighbors to affect
    animation_ms: u32,     // Animation duration
    current_hover: Option<usize>,
}

impl MagnificationController {
    pub fn calculate_scale(&self, item_index: usize, cursor_x: f64) -> f64 {
        // Cosine-based magnification
    }
}
```

### Task 1.3: Improved Hover Effects
**Priority:** Medium | **Effort:** 3 hours

- Smooth scale transitions
- Glow effect on hover
- Icon bounce on click

### Task 1.4: Settings Dialog
**Priority:** Medium | **Effort:** 6 hours

```rust
pub struct SettingsDialog {
    window: gtk::Window,
    position_combo: gtk::ComboBoxText,
    icon_size_scale: gtk::Scale,
    opacity_scale: gtk::Scale,
    auto_hide_switch: gtk::Switch,
}
```

---

## Sprint 2: Window Integration Foundation (Week 3-4)

### Task 2.1: Foreign Toplevel Protocol
**Priority:** Critical | **Effort:** 12 hours

**Purpose:** Track all open windows on Wayland

```rust
// Window tracking service
pub struct WindowTracker {
    windows: HashMap<u32, WindowInfo>,
    app_windows: HashMap<String, Vec<u32>>,
}

pub struct WindowInfo {
    id: u32,
    app_id: String,
    title: String,
    state: WindowState,
    output: Option<String>,
}
```

**Wayland Protocol:** `zwlr_foreign_toplevel_management_v1`

**Dependencies to add:**
```toml
wayland-client = "0.31"
wayland-protocols-wlr = "0.3"
```

### Task 2.2: Running App Detection
**Priority:** High | **Effort:** 4 hours

Connect window tracker to dock items:
- Update dot indicators
- Show/hide running state
- Track focused window

### Task 2.3: Window Count Badges
**Priority:** Medium | **Effort:** 4 hours

Show number of windows when >1:
- Small badge in corner
- Update on window open/close

---

## Sprint 3: Visual Enhancements (Week 5-6)

### Task 3.1: Badge System
**Priority:** High | **Effort:** 8 hours

```rust
pub enum Badge {
    Count(u32),                    // Notification count
    Progress(f64),                 // 0.0 - 1.0
    Attention,                     // Urgent indicator
    Custom { icon: String },       // Custom icon badge
}

pub struct BadgeRenderer {
    badge_type: Badge,
    position: BadgePosition,       // TopRight, BottomRight, etc.
}
```

### Task 3.2: Progress Rings
**Priority:** High | **Effort:** 6 hours

Cairo-drawn circular progress:
- Determinate progress (ring fills)
- Indeterminate (spinning)
- Paused state
- Error state (red)

### Task 3.3: Theme Detection
**Priority:** Medium | **Effort:** 6 hours

```rust
pub struct ThemeManager {
    current_theme: String,
    accent_color: gdk::RGBA,
    is_dark: bool,
}

impl ThemeManager {
    pub fn detect_system_theme() -> Self {
        // Read from GSettings
        // org.gnome.desktop.interface gtk-theme
        // org.gnome.desktop.interface color-scheme
    }
    
    pub fn watch_for_changes(&self, callback: impl Fn(&Self)) {
        // Monitor GSettings changes
    }
}
```

---

## Sprint 4: D-Bus Integration (Week 7-8)

### Task 4.1: D-Bus Server Setup
**Priority:** Critical | **Effort:** 10 hours

```rust
use zbus::{ConnectionBuilder, dbus_interface};

struct DockInterface {
    dock: Arc<Mutex<DockState>>,
}

#[dbus_interface(name = "com.blazedock.Dock")]
impl DockInterface {
    fn add_pinned_app(&self, desktop_file: &str) -> bool;
    fn remove_pinned_app(&self, app_id: &str) -> bool;
    fn set_badge(&self, app_id: &str, badge_type: &str, value: i32);
    fn set_progress(&self, app_id: &str, progress: f64);
    
    #[dbus_interface(signal)]
    fn app_launched(&self, app_id: &str);
}
```

### Task 4.2: Notification Listener
**Priority:** High | **Effort:** 6 hours

Monitor `org.freedesktop.Notifications`:
- Track notification count per app
- Store recent notifications
- Update badges in real-time

### Task 4.3: Unity LauncherEntry Support
**Priority:** Medium | **Effort:** 4 hours

Support existing Linux apps that use Unity protocol:
- Firefox download progress
- File managers
- IDEs

---

## Sprint 5: Window Previews (Week 9-10)

### Task 5.1: Screencopy Protocol
**Priority:** High | **Effort:** 8 hours

**Protocol:** `wlr-screencopy-unstable-v1`

```rust
pub struct ScreenCapture {
    // Capture specific window or output
    pub async fn capture_window(window_id: u32) -> Result<gdk::Texture>;
    pub async fn capture_output(output: &str) -> Result<gdk::Texture>;
}
```

### Task 5.2: Preview Popover
**Priority:** High | **Effort:** 8 hours

```rust
pub struct PreviewPopover {
    popover: gtk::Popover,
    preview_grid: gtk::Grid,
    windows: Vec<WindowPreview>,
}

impl PreviewPopover {
    pub fn show_for_app(&self, app_id: &str) {
        // Get windows for app
        // Capture thumbnails
        // Display in grid
    }
}
```

### Task 5.3: Live Preview Updates
**Priority:** Medium | **Effort:** 4 hours

- Periodic thumbnail refresh
- Efficient caching
- Lazy loading

---

## Sprint 6: Intelligence Features (Week 11-12)

### Task 6.1: Auto-Hide Logic
**Priority:** High | **Effort:** 6 hours

```rust
pub struct AutoHideController {
    enabled: bool,
    delay_ms: u32,
    fullscreen_apps: HashSet<String>,
    exceptions: Vec<String>,  // Apps to never hide for
}

impl AutoHideController {
    pub fn should_hide(&self, context: &HideContext) -> bool {
        // Check fullscreen state
        // Check exceptions
        // Check notification state
    }
}
```

### Task 6.2: Edge Unhide Detection
**Priority:** Medium | **Effort:** 4 hours

- Monitor pointer position near edge
- Show dock on hover at edge
- Configurable trigger zone

### Task 6.3: App State Detection
**Priority:** Low | **Effort:** 6 hours

```rust
pub struct AppHealth {
    pub is_responding: bool,
    pub cpu_percent: f32,
    pub memory_mb: u32,
    pub is_playing_audio: bool,
}
```

---

## Sprint 7: Keyboard & Shortcuts (Week 13-14)

### Task 7.1: Global Shortcuts
**Priority:** High | **Effort:** 8 hours

Register with desktop environment:
- GNOME: `org.gnome.Shell` D-Bus
- KDE: KGlobalAccel D-Bus

```rust
pub struct ShortcutManager {
    shortcuts: HashMap<String, Action>,
}

pub enum Action {
    LaunchOrFocus(usize),  // Super+1-9
    NewWindow(usize),       // Super+Shift+1-9
    ToggleDock,            // Super+D
    SearchDock,            // Super+/
}
```

### Task 7.2: Keyboard Navigation
**Priority:** Medium | **Effort:** 4 hours

- Arrow keys to navigate
- Enter to activate
- Menu key for context menu

### Task 7.3: Type-to-Search
**Priority:** Medium | **Effort:** 4 hours

- Filter dock items by typing
- Highlight matching apps
- Quick launch on Enter

---

## Sprint 8: Multi-Monitor & Polish (Week 15-16)

### Task 8.1: Multi-Monitor Support
**Priority:** High | **Effort:** 10 hours

```rust
pub enum MultiMonitorMode {
    PrimaryOnly,
    AllMonitors,
    FollowMouse,
    Custom(Vec<MonitorConfig>),
}

pub struct MonitorConfig {
    output_name: String,
    enabled: bool,
    position: DockPosition,
}
```

### Task 8.2: Profile System
**Priority:** Medium | **Effort:** 6 hours

- Multiple named profiles
- Quick profile switching
- Auto-switch based on context

### Task 8.3: Performance Optimization
**Priority:** High | **Effort:** 8 hours

- Profile with `perf`
- Optimize icon loading
- Reduce memory allocations
- Improve animation performance

---

## Dependency Updates for Cargo.toml

```toml
# Phase 2-3: Window Management
wayland-client = "0.31"
wayland-protocols-wlr = "0.3"

# Phase 4: D-Bus Integration  
zbus = "4.0"

# Phase 5: Intelligence
procfs = "0.16"
rusqlite = { version = "0.31", features = ["bundled"] }

# Phase 6: Advanced
mlua = { version = "0.9", features = ["lua54", "vendored"] }  # Optional: Lua scripting

# Utilities
palette = "0.7"      # Color manipulation
image = "0.25"       # Image processing
```

---

## Testing Strategy

### Unit Tests
- Configuration parsing
- Magnification calculations
- Badge rendering
- Icon resolution

### Integration Tests
- D-Bus API
- Window tracking
- Theme detection
- File operations

### Manual Testing Checklist
- [ ] KDE Plasma 6
- [ ] GNOME 45+
- [ ] Sway
- [ ] Hyprland
- [ ] Multi-monitor setups
- [ ] HiDPI displays
- [ ] Different icon themes

---

## Success Metrics

### Phase 1 Complete When:
- Dock anchors correctly on all tested compositors
- Running indicators work
- Basic magnification implemented
- Settings dialog functional

### Phase 2 Complete When:
- Window tracking accurate
- Badges display correctly
- Theme auto-detection works
- 60fps animations achieved

### Phase 3 Complete When:
- D-Bus API documented and functional
- Apps can update their own badges
- Notification counts accurate
- Preview thumbnails working

### Final Release Criteria:
- All performance targets met
- Documentation complete
- Packaged for Fedora (RPM)
- CI/CD pipeline operational

