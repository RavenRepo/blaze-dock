//! UI module
//!
//! Contains all user interface components including the main dock window,
//! dock items, and styling.

mod window;
mod dock_item;
mod style;

pub use window::DockWindow;
pub use dock_item::DockItem;
pub use style::load_global_styles;

