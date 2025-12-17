//! UI module
//!
//! Contains all user interface components including the main dock window,
//! dock items, and styling.

mod window;
mod dock_item;
mod style;
mod running_indicator;
mod magnification;
mod settings_dialog;
mod badge;
mod window_preview;

pub use window::DockWindow;
pub use dock_item::DockItem;
pub use style::load_global_styles;
pub use running_indicator::{RunningIndicator, RunningState};
pub use magnification::MagnificationController;
pub use settings_dialog::SettingsDialog;
pub use badge::{Badge, BadgeType, BadgePosition};
pub use window_preview::WindowPreview;

