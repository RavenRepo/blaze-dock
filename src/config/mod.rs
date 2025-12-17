//! Configuration module
//!
//! Handles loading, saving, and managing BlazeDock settings.

mod settings;
pub mod profiles;

pub use settings::Settings;
pub use settings::DockPosition;
pub use settings::PinnedApp;
pub use settings::MultiMonitorMode;
pub use profiles::{Profile, ProfileManager, ProfileMeta};

