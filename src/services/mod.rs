//! Services module
//!
//! Background services for window tracking, notifications, etc.

pub mod process_tracker;
pub mod dbus_service;
pub mod window_tracker;
pub mod drive_monitor;
pub mod recent_files;
pub mod running_apps;
pub mod theme_service;
pub mod keyboard_service;
pub mod multimonitor;
pub mod screencopy_service;

pub use process_tracker::ProcessTracker;
pub use dbus_service::DBusService;
pub use window_tracker::WindowTracker;
pub use drive_monitor::DriveMonitor;
pub use recent_files::RecentFilesService;
pub use running_apps::{RunningAppsService, RunningApp};
pub use theme_service::{ThemeService, ThemeColors, ThemeMode};
pub use keyboard_service::{KeyboardService, ShortcutAction, ShortcutBinding};
pub use multimonitor::{MultiMonitorService, MonitorInfo, MultiMonitorMode};
pub use screencopy_service::{ScreencopyService, WindowThumbnail};

