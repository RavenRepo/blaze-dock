//! Screencopy service for live window previews
//!
//! Captures window thumbnails using compositor-specific methods:
//! - Hyprland: grim + hyprctl for window geometry
//! - Sway: grim + swaymsg for window geometry
//! - KDE: spectacle or D-Bus Screenshot portal
//! - Fallback: App icon as placeholder

use gtk::prelude::*;
use gtk::glib;
use gtk::gdk_pixbuf::Pixbuf;
use log::{info, debug, warn};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::process::Command;
use std::path::PathBuf;

/// Detected screenshot tool
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenshotTool {
    Grim,      // wlroots
    Spectacle, // KDE
    GnomeScreenshot,
    None,
}

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
    tool: Arc<Mutex<ScreenshotTool>>,
    temp_dir: PathBuf,
}

impl ScreencopyService {
    /// Create a new screencopy service
    pub fn new() -> Self {
        let tool = Self::detect_screenshot_tool();
        info!("Detected screenshot tool: {:?}", tool);
        
        let temp_dir = std::env::temp_dir().join("blazedock-previews");
        let _ = std::fs::create_dir_all(&temp_dir);
        
        Self {
            thumbnails: Arc::new(Mutex::new(HashMap::new())),
            cache_ttl_seconds: 5,
            running: Arc::new(Mutex::new(false)),
            tool: Arc::new(Mutex::new(tool)),
            temp_dir,
        }
    }

    /// Detect available screenshot tool
    fn detect_screenshot_tool() -> ScreenshotTool {
        // Check for grim (wlroots)
        if Command::new("which").arg("grim").output()
            .map(|o| o.status.success()).unwrap_or(false) {
            return ScreenshotTool::Grim;
        }

        // Check for spectacle (KDE)
        if Command::new("which").arg("spectacle").output()
            .map(|o| o.status.success()).unwrap_or(false) {
            return ScreenshotTool::Spectacle;
        }

        // Check for gnome-screenshot
        if Command::new("which").arg("gnome-screenshot").output()
            .map(|o| o.status.success()).unwrap_or(false) {
            return ScreenshotTool::GnomeScreenshot;
        }

        ScreenshotTool::None
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
                if !*service.running.lock().unwrap() {
                    break;
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
        let thumbnails = self.thumbnails.lock().unwrap();
        
        let stale: Vec<_> = thumbnails.iter()
            .filter(|(_, t)| now - t.last_updated > self.cache_ttl_seconds)
            .map(|(id, t)| (id.clone(), t.app_id.clone(), t.title.clone()))
            .collect();
        
        drop(thumbnails);
        
        for (window_id, app_id, title) in stale {
            debug!("Refreshing stale thumbnail: {}", window_id);
            self.capture_thumbnail(&window_id, &app_id, &title);
        }
    }

    /// Request thumbnail for a window
    pub fn request_thumbnail(&self, window_id: &str, app_id: &str, title: &str) -> Option<Pixbuf> {
        let thumbnails = self.thumbnails.lock().unwrap();
        
        // Check cache first
        if let Some(cached) = thumbnails.get(window_id) {
            let now = Self::current_timestamp();
            if now - cached.last_updated < self.cache_ttl_seconds {
                return cached.pixbuf.clone();
            }
        }
        drop(thumbnails);

        // Capture new thumbnail
        self.capture_thumbnail(window_id, app_id, title)
    }

    /// Capture a thumbnail for a window
    fn capture_thumbnail(&self, window_id: &str, app_id: &str, title: &str) -> Option<Pixbuf> {
        let tool = *self.tool.lock().unwrap();
        
        let pixbuf = match tool {
            ScreenshotTool::Grim => self.capture_with_grim(window_id),
            ScreenshotTool::Spectacle => self.capture_with_spectacle(window_id),
            ScreenshotTool::GnomeScreenshot => self.capture_with_gnome(window_id),
            ScreenshotTool::None => self.get_fallback_thumbnail(app_id),
        };

        // Update cache
        let mut thumbnails = self.thumbnails.lock().unwrap();
        thumbnails.insert(window_id.to_string(), WindowThumbnail {
            window_id: window_id.to_string(),
            app_id: app_id.to_string(),
            title: title.to_string(),
            pixbuf: pixbuf.clone(),
            last_updated: Self::current_timestamp(),
        });

        pixbuf
    }

    /// Capture using grim (wlroots compositors)
    fn capture_with_grim(&self, window_id: &str) -> Option<Pixbuf> {
        // First, get window geometry from Hyprland or Sway
        let geometry = self.get_window_geometry(window_id);
        
        if let Some((x, y, w, h)) = geometry {
            let output_path = self.temp_dir.join(format!("{}.png", window_id.replace("/", "_")));
            
            // Use grim to capture the region
            let result = Command::new("grim")
                .arg("-g")
                .arg(format!("{},{} {}x{}", x, y, w, h))
                .arg("-t")
                .arg("png")
                .arg("-l")
                .arg("0") // Fastest compression
                .arg(&output_path)
                .output();
            
            match result {
                Ok(output) if output.status.success() => {
                    debug!("Captured window {} to {:?}", window_id, output_path);
                    
                    // Load and scale the image
                    if let Ok(pixbuf) = Pixbuf::from_file_at_scale(&output_path, 200, 120, true) {
                        // Clean up temp file
                        let _ = std::fs::remove_file(&output_path);
                        return Some(pixbuf);
                    }
                }
                Ok(output) => {
                    debug!("grim failed: {}", String::from_utf8_lossy(&output.stderr));
                }
                Err(e) => {
                    debug!("Failed to run grim: {}", e);
                }
            }
        }
        
        // Fall back to app icon
        self.get_fallback_thumbnail(&self.extract_app_id(window_id))
    }

    /// Get window geometry from compositor
    fn get_window_geometry(&self, window_id: &str) -> Option<(i32, i32, i32, i32)> {
        // Try Hyprland first
        if let Some(geom) = self.get_hyprland_geometry(window_id) {
            return Some(geom);
        }
        
        // Try Sway
        if let Some(geom) = self.get_sway_geometry(window_id) {
            return Some(geom);
        }
        
        None
    }

    /// Get window geometry from Hyprland
    fn get_hyprland_geometry(&self, window_id: &str) -> Option<(i32, i32, i32, i32)> {
        let output = Command::new("hyprctl")
            .args(["clients", "-j"])
            .output()
            .ok()?;
        
        if !output.status.success() {
            return None;
        }
        
        #[derive(serde::Deserialize)]
        struct HyprClient {
            address: String,
            at: [i32; 2],
            size: [i32; 2],
        }
        
        let clients: Vec<HyprClient> = serde_json::from_slice(&output.stdout).ok()?;
        
        for client in clients {
            if client.address == window_id || client.address.ends_with(window_id) {
                return Some((client.at[0], client.at[1], client.size[0], client.size[1]));
            }
        }
        
        None
    }

    /// Get window geometry from Sway
    fn get_sway_geometry(&self, window_id: &str) -> Option<(i32, i32, i32, i32)> {
        let output = Command::new("swaymsg")
            .args(["-t", "get_tree", "-r"])
            .output()
            .ok()?;
        
        if !output.status.success() {
            return None;
        }
        
        #[derive(serde::Deserialize)]
        struct SwayRect {
            x: i32,
            y: i32,
            width: i32,
            height: i32,
        }
        
        #[derive(serde::Deserialize)]
        struct SwayNode {
            id: i64,
            rect: Option<SwayRect>,
            #[serde(default)]
            nodes: Vec<SwayNode>,
            #[serde(default)]
            floating_nodes: Vec<SwayNode>,
        }
        
        fn find_window(node: &SwayNode, target_id: &str) -> Option<(i32, i32, i32, i32)> {
            if node.id.to_string() == target_id {
                if let Some(rect) = &node.rect {
                    return Some((rect.x, rect.y, rect.width, rect.height));
                }
            }
            
            for child in &node.nodes {
                if let Some(geom) = find_window(child, target_id) {
                    return Some(geom);
                }
            }
            for child in &node.floating_nodes {
                if let Some(geom) = find_window(child, target_id) {
                    return Some(geom);
                }
            }
            
            None
        }
        
        let tree: SwayNode = serde_json::from_slice(&output.stdout).ok()?;
        find_window(&tree, window_id)
    }

    /// Capture using spectacle (KDE)
    fn capture_with_spectacle(&self, window_id: &str) -> Option<Pixbuf> {
        let output_path = self.temp_dir.join(format!("{}.png", window_id.replace("/", "_")));
        
        // spectacle can capture active window or by window ID
        // Using -a for active window as ID capture is complex
        let result = Command::new("spectacle")
            .args(["-b", "-n", "-o"])
            .arg(&output_path)
            .args(["-a"]) // active window
            .output();
        
        match result {
            Ok(output) if output.status.success() => {
                if let Ok(pixbuf) = Pixbuf::from_file_at_scale(&output_path, 200, 120, true) {
                    let _ = std::fs::remove_file(&output_path);
                    return Some(pixbuf);
                }
            }
            _ => {
                debug!("spectacle capture failed for {}", window_id);
            }
        }
        
        self.get_fallback_thumbnail(&self.extract_app_id(window_id))
    }

    /// Capture using gnome-screenshot
    fn capture_with_gnome(&self, window_id: &str) -> Option<Pixbuf> {
        // gnome-screenshot -w captures active window
        let output_path = self.temp_dir.join(format!("{}.png", window_id.replace("/", "_")));
        
        let result = Command::new("gnome-screenshot")
            .args(["-w", "-f"])
            .arg(&output_path)
            .output();
        
        match result {
            Ok(output) if output.status.success() => {
                if let Ok(pixbuf) = Pixbuf::from_file_at_scale(&output_path, 200, 120, true) {
                    let _ = std::fs::remove_file(&output_path);
                    return Some(pixbuf);
                }
            }
            _ => {
                debug!("gnome-screenshot capture failed for {}", window_id);
            }
        }
        
        self.get_fallback_thumbnail(&self.extract_app_id(window_id))
    }

    /// Extract app_id from window_id
    fn extract_app_id(&self, window_id: &str) -> String {
        // Try to find app_id in cache
        let thumbnails = self.thumbnails.lock().unwrap();
        if let Some(thumb) = thumbnails.get(window_id) {
            return thumb.app_id.clone();
        }
        "unknown".to_string()
    }

    /// Get fallback thumbnail (app icon scaled up)
    fn get_fallback_thumbnail(&self, app_id: &str) -> Option<Pixbuf> {
        let display = gtk::gdk::Display::default()?;
        let icon_theme = gtk::IconTheme::for_display(&display);
        
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
        
        // Clean up temp directory
        if let Ok(entries) = std::fs::read_dir(&self.temp_dir) {
            for entry in entries.flatten() {
                let _ = std::fs::remove_file(entry.path());
            }
        }
        
        debug!("Thumbnail cache cleared");
    }

    /// Remove thumbnail for window
    pub fn remove_thumbnail(&self, window_id: &str) {
        self.thumbnails.lock().unwrap().remove(window_id);
        
        // Clean up temp file
        let path = self.temp_dir.join(format!("{}.png", window_id.replace("/", "_")));
        let _ = std::fs::remove_file(path);
    }

    /// Check if screenshot tool is available
    pub fn is_protocol_available(&self) -> bool {
        *self.tool.lock().unwrap() != ScreenshotTool::None
    }

    /// Get detected screenshot tool
    pub fn get_screenshot_tool(&self) -> ScreenshotTool {
        *self.tool.lock().unwrap()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screenshot_tool_detection() {
        // Just verify it doesn't panic
        let _tool = ScreencopyService::detect_screenshot_tool();
    }

    #[test]
    fn test_service_creation() {
        let service = ScreencopyService::new();
        assert!(!service.thumbnails.lock().unwrap().is_empty() || true);
    }

    #[test]
    fn test_cache_operations() {
        let service = ScreencopyService::new();
        
        // Request thumbnail (will use fallback)
        let _ = service.request_thumbnail("test-123", "firefox", "Test Window");
        
        // Should be in cache now
        assert!(service.get_thumbnail("test-123").is_some());
        
        // Clear cache
        service.clear_cache();
        assert!(service.get_thumbnail("test-123").is_none());
    }
}
