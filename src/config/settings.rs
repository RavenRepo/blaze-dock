//! Settings management for BlazeDock
//!
//! Defines the configuration structure and handles loading/saving
//! settings to a TOML file.

use anyhow::{Context, Result};
use directories::ProjectDirs;
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Configuration file name
const CONFIG_FILE: &str = "blazedock.toml";

/// Dock position on screen
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum DockPosition {
    #[default]
    Left,
    Right,
    Top,
    Bottom,
}

/// A pinned application entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinnedApp {
    /// Display name of the application
    pub name: String,
    /// Icon name (from theme) or path to icon file
    pub icon: String,
    /// Command to execute when clicked
    pub command: String,
    /// Optional .desktop file path for richer integration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub desktop_file: Option<String>,
}

/// Multi-monitor mode
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum MultiMonitorMode {
    #[default]
    Primary,
    All,
    Follow,
    PerMonitor,
}

/// Main settings structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Dock position on screen
    pub position: DockPosition,
    
    /// Icon size in pixels
    pub icon_size: u32,
    
    /// Dock width/height (depending on orientation)
    pub dock_size: u32,
    
    /// Margin from screen edge
    pub margin: u32,
    
    /// Spacing between icons
    pub spacing: u32,
    
    /// Enable auto-hide behavior
    pub auto_hide: bool,
    
    /// Auto-hide delay in milliseconds
    pub auto_hide_delay: u32,
    
    /// Background opacity (0.0 - 1.0)
    pub opacity: f64,
    
    /// Border radius for rounded corners
    pub border_radius: u32,
    
    /// Enable exclusive zone (push windows aside)
    pub exclusive_zone: bool,
    
    /// Enable hover zoom effect
    pub hover_zoom: bool,
    
    /// Hover zoom scale factor
    pub hover_zoom_scale: f64,
    
    /// Multi-monitor mode
    pub multi_monitor_mode: MultiMonitorMode,
    
    /// Enable keyboard shortcuts (Super+1-9)
    pub enable_shortcuts: bool,
    
    /// Active profile name
    pub active_profile: String,
    
    /// Enable dynamic running apps display
    pub show_running_apps: bool,
    
    /// Enable window previews on hover
    pub enable_window_previews: bool,
    
    /// Theme mode (light/dark/system)
    pub theme_mode: String,
    
    /// Show trash icon at end of dock
    pub show_trash: bool,
    
    /// Show Downloads stack at end of dock
    pub show_downloads_stack: bool,
    
    /// List of pinned applications
    pub pinned_apps: Vec<PinnedApp>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            position: DockPosition::Bottom,
            icon_size: 48,
            dock_size: 72,
            margin: 8,
            spacing: 8,
            auto_hide: false,
            auto_hide_delay: 500,
            opacity: 0.85,
            border_radius: 16,
            exclusive_zone: false,
            hover_zoom: true,
            hover_zoom_scale: 1.15,
            multi_monitor_mode: MultiMonitorMode::Primary,
            enable_shortcuts: true,
            active_profile: "default".to_string(),
            show_running_apps: true,
            enable_window_previews: true,
            theme_mode: "system".to_string(),
            show_trash: true,
            show_downloads_stack: true,
            pinned_apps: Self::default_pinned_apps(),
        }
    }
}

impl Settings {
    /// Get the configuration file path
    pub fn config_path() -> Option<PathBuf> {
        ProjectDirs::from("com", "blazedock", "BlazeDock")
            .map(|dirs| dirs.config_dir().join(CONFIG_FILE))
    }

    /// Load settings from the configuration file
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()
            .context("Failed to determine config directory")?;
        
        debug!("Loading configuration from: {:?}", config_path);

        if !config_path.exists() {
            debug!("Config file not found, creating default configuration");
            let settings = Self::default();
            settings.save()?;
            return Ok(settings);
        }

        let content = fs::read_to_string(&config_path)
            .context("Failed to read config file")?;
        
        let settings: Settings = toml::from_str(&content)
            .context("Failed to parse config file")?;
        
        Ok(settings)
    }

    /// Save settings to the configuration file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()
            .context("Failed to determine config directory")?;
        
        // Create config directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }

        let content = toml::to_string_pretty(self)
            .context("Failed to serialize settings")?;
        
        fs::write(&config_path, content)
            .context("Failed to write config file")?;
        
        debug!("Configuration saved to: {:?}", config_path);
        Ok(())
    }

    /// Get default pinned applications
    fn default_pinned_apps() -> Vec<PinnedApp> {
        vec![
            PinnedApp {
                name: "Firefox".to_string(),
                icon: "firefox".to_string(),
                command: "firefox".to_string(),
                desktop_file: Some("/usr/share/applications/firefox.desktop".to_string()),
            },
            PinnedApp {
                name: "Files".to_string(),
                icon: "org.gnome.Nautilus".to_string(),
                command: "nautilus".to_string(),
                desktop_file: Some("/usr/share/applications/org.gnome.Nautilus.desktop".to_string()),
            },
            PinnedApp {
                name: "Terminal".to_string(),
                icon: "org.gnome.Terminal".to_string(),
                command: "gnome-terminal".to_string(),
                desktop_file: Some("/usr/share/applications/org.gnome.Terminal.desktop".to_string()),
            },
            PinnedApp {
                name: "Settings".to_string(),
                icon: "org.gnome.Settings".to_string(),
                command: "gnome-control-center".to_string(),
                desktop_file: Some("/usr/share/applications/org.gnome.Settings.desktop".to_string()),
            },
        ]
    }

    /// Add a pinned application
    pub fn add_pinned_app(&mut self, app: PinnedApp) {
        self.pinned_apps.push(app);
        if let Err(e) = self.save() {
            warn!("Failed to save config after adding app: {}", e);
        }
    }

    /// Remove a pinned application by index
    pub fn remove_pinned_app(&mut self, index: usize) -> Option<PinnedApp> {
        if index < self.pinned_apps.len() {
            let removed = self.pinned_apps.remove(index);
            if let Err(e) = self.save() {
                warn!("Failed to save config after removing app: {}", e);
            }
            Some(removed)
        } else {
            None
        }
    }

    /// Reorder a pinned application
    pub fn reorder_pinned_app(&mut self, from: usize, to: usize) {
        if from < self.pinned_apps.len() && to < self.pinned_apps.len() {
            let app = self.pinned_apps.remove(from);
            self.pinned_apps.insert(to, app);
            if let Err(e) = self.save() {
                warn!("Failed to save config after reordering: {}", e);
            }
        }
    }
}

