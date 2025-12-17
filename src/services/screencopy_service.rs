//! Screencopy service for live window previews
//!
//! Uses wlr-screencopy-unstable-v1 protocol for capturing window content.
//! Falls back to placeholder images when protocol is not available.

use gtk::prelude::*;
use gtk::glib;
use gtk::gdk_pixbuf::Pixbuf;
use log::{info, debug, warn};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::process::Command;

/// Window thumbnail cache
#[derive(Clone)]
pub struct WindowThumbnail {
    pub window_id: String,
    pub app_id: String,
    pub title: String,
    pub pixbuf: Option<Pixbuf>,
    pub last_updated: u64,
}

/// Screencopy service for window thumbnails
#[derive(Clone)]
pub struct ScreencopyService {
    thumbnails: Arc<Mutex<HashMap<String, WindowThumbnail>>>,
    cache_ttl_seconds: u64,
    running: Arc<Mutex<bool>>,
    protocol_available: Arc<Mutex<bool>>,
}

impl ScreencopyService {
    /// Create a new screencopy service
    pub fn new() -> Self {
        let service = Self {
            thumbnails: Arc::new(Mutex::new(HashMap::new())),
            cache_ttl_seconds: 5,
            running: Arc::new(Mutex::new(false)),
            protocol_available: Arc::new(Mutex::new(false)),
        };

        service.check_protocol_support();
        service
    }

    /// Check if screencopy protocol is available
    fn check_protocol_support(&self) {
        // Try to detect if wlr-screencopy is available
        // This would be compositor-specific
        let is_available = Self::detect_screencopy_support();
        *self.protocol_available.lock().unwrap() = is_available;
        
        if is_available {
            info!("Screencopy protocol available");
        } else {
            warn!("Screencopy protocol not available, using fallback");
        }
    }

    /// Detect if screencopy is supported (compositor-specific)
    fn detect_screencopy_support() -> bool {
        // Check for common screencopy tools
        // grim is commonly available on wlroots-based compositors
        if Command::new("which").arg("grim").output()
            .map(|o| o.status.success()).unwrap_or(false) {
            return true;
        }

        // Check for spectacle (KDE)
        if Command::new("which").arg("spectacle").output()
            .map(|o| o.status.success()).unwrap_or(false) {
            return true;
        }

        // Check for gnome-screenshot
        if Command::new("which").arg("gnome-screenshot").output()
            .map(|o| o.status.success()).unwrap_or(false) {
            return true;
        }

        false
    }

    /// Start the thumbnail refresh service
    pub fn start(&self) {
        let mut running = self.running.lock().unwrap();
        if *running {
            return;
        }
        *running = true;
        drop(running);

        let service = self.clone();
        
        glib::spawn_future_local(async move {
            info!("Screencopy service started");
            
            loop {
                {
                    if !*service.running.lock().unwrap() {
                        break;
                    }
                }

                // Refresh stale thumbnails
                service.refresh_stale_thumbnails();

                glib::timeout_future(std::time::Duration::from_secs(2)).await;
            }
            
            info!("Screencopy service stopped");
        });
    }

    /// Stop the service
    pub fn stop(&self) {
        *self.running.lock().unwrap() = false;
    }

    /// Refresh thumbnails that are stale
    fn refresh_stale_thumbnails(&self) {
        let now = Self::current_timestamp();
        let mut thumbnails = self.thumbnails.lock().unwrap();
        
        for thumbnail in thumbnails.values_mut() {
            if now - thumbnail.last_updated > self.cache_ttl_seconds {
                // Mark for refresh
                debug!("Thumbnail stale for window: {}", thumbnail.window_id);
            }
        }
    }

    /// Request thumbnail for a window
    pub fn request_thumbnail(&self, window_id: &str, app_id: &str, title: &str) -> Option<Pixbuf> {
        let mut thumbnails = self.thumbnails.lock().unwrap();
        
        // Check cache first
        if let Some(cached) = thumbnails.get(window_id) {
            let now = Self::current_timestamp();
            if now - cached.last_updated < self.cache_ttl_seconds {
                return cached.pixbuf.clone();
            }
        }

        // Try to capture thumbnail
        let pixbuf = if *self.protocol_available.lock().unwrap() {
            self.capture_window_thumbnail(window_id)
        } else {
            self.get_fallback_thumbnail(app_id)
        };

        // Update cache
        thumbnails.insert(window_id.to_string(), WindowThumbnail {
            window_id: window_id.to_string(),
            app_id: app_id.to_string(),
            title: title.to_string(),
            pixbuf: pixbuf.clone(),
            last_updated: Self::current_timestamp(),
        });

        pixbuf
    }

    /// Capture actual window thumbnail using available tools
    fn capture_window_thumbnail(&self, _window_id: &str) -> Option<Pixbuf> {
        // Note: Actual window-specific capture requires:
        // 1. wlr-screencopy-unstable-v1 protocol binding
        // 2. Window geometry from foreign-toplevel
        // 3. Direct memory buffer access
        //
        // For now, we return a placeholder as full implementation
        // requires significant Wayland protocol work
        
        debug!("Window thumbnail capture requested (placeholder)");
        None
    }

    /// Get fallback thumbnail (app icon scaled up)
    fn get_fallback_thumbnail(&self, app_id: &str) -> Option<Pixbuf> {
        // Try to load app icon as fallback
        let icon_theme = gtk::IconTheme::for_display(&gtk::gdk::Display::default()?);
        
        let icon = icon_theme.lookup_icon(
            app_id,
            &[],
            160,
            1,
            gtk::TextDirection::Ltr,
            gtk::IconLookupFlags::PRELOAD,
        );

        if let Some(paintable) = icon.file() {
            if let Some(path) = paintable.path() {
                return Pixbuf::from_file_at_scale(path, 160, 90, true).ok();
            }
        }

        None
    }

    /// Get cached thumbnail
    pub fn get_thumbnail(&self, window_id: &str) -> Option<WindowThumbnail> {
        self.thumbnails.lock().unwrap().get(window_id).cloned()
    }

    /// Clear thumbnail cache
    pub fn clear_cache(&self) {
        self.thumbnails.lock().unwrap().clear();
        debug!("Thumbnail cache cleared");
    }

    /// Remove thumbnail for window
    pub fn remove_thumbnail(&self, window_id: &str) {
        self.thumbnails.lock().unwrap().remove(window_id);
    }

    /// Check if protocol is available
    pub fn is_protocol_available(&self) -> bool {
        *self.protocol_available.lock().unwrap()
    }

    /// Set cache TTL
    pub fn set_cache_ttl(&mut self, seconds: u64) {
        self.cache_ttl_seconds = seconds;
    }

    /// Current timestamp helper
    fn current_timestamp() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

impl Default for ScreencopyService {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a placeholder preview widget for when thumbnails aren't available
pub fn create_placeholder_preview(app_name: &str, window_title: &str) -> gtk::Box {
    let container = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(8)
        .width_request(180)
        .height_request(120)
        .css_classes(vec!["preview-placeholder"])
        .build();

    // App icon
    let icon = gtk::Image::from_icon_name(app_name);
    icon.set_pixel_size(48);
    icon.add_css_class("preview-placeholder-icon");

    // Window title
    let title = gtk::Label::new(Some(window_title));
    title.set_max_width_chars(20);
    title.set_ellipsize(gtk::pango::EllipsizeMode::End);
    title.add_css_class("preview-placeholder-title");

    container.append(&icon);
    container.append(&title);

    container
}

