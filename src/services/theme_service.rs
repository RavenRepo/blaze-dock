//! Theme detection and auto-matching service
//!
//! Monitors system theme changes and provides color information for theming.

use gtk::glib;
use gtk::Settings as GtkSettings;
use log::{info, debug};
use std::sync::{Arc, Mutex};

/// Theme mode (light/dark)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Light,
    Dark,
    System, // Follow system preference
}

/// Theme colors extracted from system
#[derive(Debug, Clone)]
pub struct ThemeColors {
    pub accent_color: (f64, f64, f64),      // RGB 0.0-1.0
    pub background_color: (f64, f64, f64),
    pub foreground_color: (f64, f64, f64),
    pub is_dark: bool,
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self {
            accent_color: (0.4, 0.6, 1.0),      // Blue accent
            background_color: (0.1, 0.1, 0.15), // Dark background
            foreground_color: (1.0, 1.0, 1.0),  // White text
            is_dark: true,
        }
    }
}

/// Theme service for detecting and responding to system theme changes
#[derive(Clone)]
pub struct ThemeService {
    current_mode: Arc<Mutex<ThemeMode>>,
    current_colors: Arc<Mutex<ThemeColors>>,
    callbacks: Arc<Mutex<Vec<Box<dyn Fn(&ThemeColors) + Send + Sync>>>>,
}

impl ThemeService {
    /// Create a new theme service
    pub fn new() -> Self {
        let service = Self {
            current_mode: Arc::new(Mutex::new(ThemeMode::System)),
            current_colors: Arc::new(Mutex::new(ThemeColors::default())),
            callbacks: Arc::new(Mutex::new(Vec::new())),
        };
        
        service.detect_initial_theme();
        service
    }

    /// Detect the initial system theme
    fn detect_initial_theme(&self) {
        if let Some(settings) = GtkSettings::default() {
            let is_dark = settings.is_gtk_application_prefer_dark_theme();
            debug!("GTK prefers dark theme: {}", is_dark);
            
            let mut colors = self.current_colors.lock().unwrap();
            colors.is_dark = is_dark;
            
            if is_dark {
                colors.background_color = (0.1, 0.1, 0.15);
                colors.foreground_color = (1.0, 1.0, 1.0);
            } else {
                colors.background_color = (0.95, 0.95, 0.95);
                colors.foreground_color = (0.1, 0.1, 0.1);
            }
            
            // Try to detect accent color from GTK settings
            self.detect_accent_color(&settings, &mut colors);
            
            info!("Theme detected: {} mode", if colors.is_dark { "dark" } else { "light" });
        }
    }

    /// Detect accent color from GTK settings
    fn detect_accent_color(&self, settings: &GtkSettings, colors: &mut ThemeColors) {
        // GTK4 doesn't expose accent color directly, so we use a heuristic
        // KDE Plasma uses org.kde.kdeglobals via GSettings
        // GNOME uses org.gnome.desktop.interface
        
        // Try KDE accent color
        if let Some(accent) = self.get_kde_accent_color() {
            colors.accent_color = accent;
            debug!("KDE accent color detected: {:?}", accent);
            return;
        }

        // Try GNOME accent color
        if let Some(accent) = self.get_gnome_accent_color() {
            colors.accent_color = accent;
            debug!("GNOME accent color detected: {:?}", accent);
            return;
        }

        // Fallback to default blue
        debug!("Using default accent color");
    }

    /// Get KDE accent color from GSettings/kdeglobals
    fn get_kde_accent_color(&self) -> Option<(f64, f64, f64)> {
        // Read from ~/.config/kdeglobals
        let config_path = dirs::config_dir()?.join("kdeglobals");
        
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            for line in content.lines() {
                if line.starts_with("AccentColor=") {
                    let color_str = line.strip_prefix("AccentColor=")?;
                    let parts: Vec<&str> = color_str.split(',').collect();
                    if parts.len() >= 3 {
                        let r = parts[0].trim().parse::<u8>().ok()? as f64 / 255.0;
                        let g = parts[1].trim().parse::<u8>().ok()? as f64 / 255.0;
                        let b = parts[2].trim().parse::<u8>().ok()? as f64 / 255.0;
                        return Some((r, g, b));
                    }
                }
            }
        }
        
        None
    }

    /// Get GNOME accent color
    fn get_gnome_accent_color(&self) -> Option<(f64, f64, f64)> {
        // GNOME 47+ has accent colors
        // Try to read from dconf/gsettings
        let output = std::process::Command::new("gsettings")
            .args(["get", "org.gnome.desktop.interface", "accent-color"])
            .output()
            .ok()?;
        
        if output.status.success() {
            let color_name = String::from_utf8_lossy(&output.stdout);
            let color_name = color_name.trim().trim_matches('\'');
            
            // Map GNOME accent color names to RGB
            match color_name {
                "blue" => return Some((0.2, 0.5, 0.9)),
                "teal" => return Some((0.2, 0.7, 0.7)),
                "green" => return Some((0.3, 0.7, 0.3)),
                "yellow" => return Some((0.9, 0.8, 0.2)),
                "orange" => return Some((0.9, 0.5, 0.2)),
                "red" => return Some((0.9, 0.3, 0.3)),
                "pink" => return Some((0.9, 0.4, 0.6)),
                "purple" => return Some((0.6, 0.4, 0.9)),
                "slate" => return Some((0.5, 0.5, 0.6)),
                _ => {}
            }
        }
        
        None
    }

    /// Start monitoring theme changes
    pub fn start_monitoring(&self) {
        let colors = Arc::clone(&self.current_colors);
        let callbacks = Arc::clone(&self.callbacks);
        let mode = Arc::clone(&self.current_mode);

        if let Some(settings) = GtkSettings::default() {
            // Monitor dark theme preference changes
            settings.connect_gtk_application_prefer_dark_theme_notify(move |s| {
                let is_dark = s.is_gtk_application_prefer_dark_theme();
                info!("Theme changed to: {}", if is_dark { "dark" } else { "light" });
                
                let mut colors_guard = colors.lock().unwrap();
                colors_guard.is_dark = is_dark;
                
                if is_dark {
                    colors_guard.background_color = (0.1, 0.1, 0.15);
                    colors_guard.foreground_color = (1.0, 1.0, 1.0);
                } else {
                    colors_guard.background_color = (0.95, 0.95, 0.95);
                    colors_guard.foreground_color = (0.1, 0.1, 0.1);
                }
                
                let colors_clone = colors_guard.clone();
                drop(colors_guard);
                
                // Notify all callbacks
                let cbs = callbacks.lock().unwrap();
                for callback in cbs.iter() {
                    callback(&colors_clone);
                }
            });
        }
        
        // Also monitor kdeglobals for KDE accent color changes
        self.watch_kde_config();
    }

    /// Watch KDE configuration file for changes
    fn watch_kde_config(&self) {
        let colors = Arc::clone(&self.current_colors);
        let callbacks = Arc::clone(&self.callbacks);
        
        glib::timeout_add_seconds_local(10, move || {
            // Periodically check for KDE accent color changes
            let config_path = match dirs::config_dir() {
                Some(p) => p.join("kdeglobals"),
                None => return glib::ControlFlow::Continue,
            };
            
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                for line in content.lines() {
                    if line.starts_with("AccentColor=") {
                        if let Some(color_str) = line.strip_prefix("AccentColor=") {
                            let parts: Vec<&str> = color_str.split(',').collect();
                            if parts.len() >= 3 {
                                if let (Ok(r), Ok(g), Ok(b)) = (
                                    parts[0].trim().parse::<u8>(),
                                    parts[1].trim().parse::<u8>(),
                                    parts[2].trim().parse::<u8>(),
                                ) {
                                    let new_accent = (
                                        r as f64 / 255.0,
                                        g as f64 / 255.0,
                                        b as f64 / 255.0,
                                    );
                                    
                                    let mut colors_guard = colors.lock().unwrap();
                                    if colors_guard.accent_color != new_accent {
                                        colors_guard.accent_color = new_accent;
                                        debug!("KDE accent color updated: {:?}", new_accent);
                                        
                                        let colors_clone = colors_guard.clone();
                                        drop(colors_guard);
                                        
                                        let cbs = callbacks.lock().unwrap();
                                        for callback in cbs.iter() {
                                            callback(&colors_clone);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            glib::ControlFlow::Continue
        });
    }

    /// Register a callback for theme changes
    pub fn on_theme_change<F>(&self, callback: F)
    where
        F: Fn(&ThemeColors) + Send + Sync + 'static,
    {
        let mut cbs = self.callbacks.lock().unwrap();
        cbs.push(Box::new(callback));
    }

    /// Get current theme colors
    pub fn get_colors(&self) -> ThemeColors {
        self.current_colors.lock().unwrap().clone()
    }

    /// Get current theme mode
    pub fn get_mode(&self) -> ThemeMode {
        *self.current_mode.lock().unwrap()
    }

    /// Check if dark mode is active
    pub fn is_dark_mode(&self) -> bool {
        self.current_colors.lock().unwrap().is_dark
    }

    /// Generate CSS variables for the current theme
    pub fn generate_css_variables(&self) -> String {
        let colors = self.current_colors.lock().unwrap();
        
        format!(
            r#"
            @define-color accent_color rgb({}, {}, {});
            @define-color bg_color rgb({}, {}, {});
            @define-color fg_color rgb({}, {}, {});
            "#,
            (colors.accent_color.0 * 255.0) as u8,
            (colors.accent_color.1 * 255.0) as u8,
            (colors.accent_color.2 * 255.0) as u8,
            (colors.background_color.0 * 255.0) as u8,
            (colors.background_color.1 * 255.0) as u8,
            (colors.background_color.2 * 255.0) as u8,
            (colors.foreground_color.0 * 255.0) as u8,
            (colors.foreground_color.1 * 255.0) as u8,
            (colors.foreground_color.2 * 255.0) as u8,
        )
    }
}

impl Default for ThemeService {
    fn default() -> Self {
        Self::new()
    }
}

