//! Window tracker service
//!
//! Tracks open windows via D-Bus interfaces for KDE, GNOME, and Hyprland.
//! Provides app-to-window mapping and window count information.

use gtk::glib;
use log::{info, debug, warn, error};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// Window information
#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub id: String,
    pub title: String,
    pub app_id: String,
    pub is_active: bool,
}

/// Detected desktop environment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopEnvironment {
    KDE,
    GNOME,
    Hyprland,
    Sway,
    Unknown,
}

/// Window tracker for monitoring open windows
/// Uses D-Bus interfaces for compositor-specific window tracking
#[derive(Clone)]
pub struct WindowTracker {
    windows: Arc<Mutex<Vec<WindowInfo>>>,
    app_window_counts: Arc<Mutex<HashMap<String, u32>>>,
    running: Arc<Mutex<bool>>,
    desktop: Arc<Mutex<DesktopEnvironment>>,
}

impl WindowTracker {
    /// Create a new window tracker
    pub fn new() -> Self {
        let desktop = Self::detect_desktop_environment();
        info!("Detected desktop environment: {:?}", desktop);
        
        Self {
            windows: Arc::new(Mutex::new(Vec::new())),
            app_window_counts: Arc::new(Mutex::new(HashMap::new())),
            running: Arc::new(Mutex::new(false)),
            desktop: Arc::new(Mutex::new(desktop)),
        }
    }

    /// Detect the current desktop environment
    fn detect_desktop_environment() -> DesktopEnvironment {
        // Check environment variables
        let xdg_desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();
        let xdg_session = std::env::var("XDG_SESSION_DESKTOP").unwrap_or_default();
        let hyprland = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").ok();
        let swaysock = std::env::var("SWAYSOCK").ok();
        
        if hyprland.is_some() {
            return DesktopEnvironment::Hyprland;
        }
        
        if swaysock.is_some() {
            return DesktopEnvironment::Sway;
        }
        
        let desktop_lower = xdg_desktop.to_lowercase();
        let session_lower = xdg_session.to_lowercase();
        
        if desktop_lower.contains("kde") || desktop_lower.contains("plasma") ||
           session_lower.contains("kde") || session_lower.contains("plasma") {
            return DesktopEnvironment::KDE;
        }
        
        if desktop_lower.contains("gnome") || session_lower.contains("gnome") {
            return DesktopEnvironment::GNOME;
        }
        
        DesktopEnvironment::Unknown
    }

    /// Start tracking windows
    pub fn start(&self) {
        let mut running = self.running.lock().unwrap();
        if *running {
            return;
        }
        *running = true;
        drop(running);

        let desktop = *self.desktop.lock().unwrap();
        info!("Window tracker starting for {:?}", desktop);

        match desktop {
            DesktopEnvironment::KDE => self.start_kde_tracking(),
            DesktopEnvironment::GNOME => self.start_gnome_tracking(),
            DesktopEnvironment::Hyprland => self.start_hyprland_tracking(),
            DesktopEnvironment::Sway => self.start_sway_tracking(),
            DesktopEnvironment::Unknown => {
                warn!("Unknown desktop environment, window tracking limited");
                info!("Window tracker running in fallback mode");
            }
        }
    }

    /// Start KDE window tracking via D-Bus
    fn start_kde_tracking(&self) {
        let tracker = self.clone();
        
        glib::spawn_future_local(async move {
            match tracker.poll_kde_windows().await {
                Ok(_) => info!("KDE window polling started"),
                Err(e) => warn!("Failed to start KDE window tracking: {}", e),
            }
        });
        
        // Start periodic polling
        let tracker = self.clone();
        glib::timeout_add_seconds_local(2, move || {
            if !tracker.is_running() {
                return glib::ControlFlow::Break;
            }
            
            let tracker_clone = tracker.clone();
            glib::spawn_future_local(async move {
                if let Err(e) = tracker_clone.poll_kde_windows().await {
                    debug!("KDE window poll error: {}", e);
                }
            });
            
            glib::ControlFlow::Continue
        });
    }

    /// Poll KDE windows via D-Bus
    async fn poll_kde_windows(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let connection = zbus::Connection::session().await?;
        
        // Call org.kde.KWin to get window list
        let message = connection
            .call_method(
                Some("org.kde.KWin"),
                "/KWin",
                Some("org.kde.KWin"),
                "queryWindowInfo",
                &(),
            )
            .await;
        
        match message {
            Ok(reply) => {
                // Parse the reply - KWin returns a list of window info variants
                debug!("Received KDE window info response");
                self.parse_kde_response(&reply)?;
            }
            Err(e) => {
                // Try alternative method: scripting interface
                debug!("KWin queryWindowInfo failed ({}), trying script method", e);
                self.poll_kde_via_script(&connection).await?;
            }
        }
        
        Ok(())
    }

    /// Parse KDE window response
    fn parse_kde_response(&self, _reply: &zbus::Message) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // KWin's window info is complex - for now, count windows per app
        // Full implementation would parse the variant structure
        debug!("KDE window response received");
        Ok(())
    }

    /// Poll KDE windows using the scripting interface
    async fn poll_kde_via_script(&self, connection: &zbus::Connection) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Use KWin scripting API to get window list
        let script = r#"
            var clients = workspace.clientList();
            var result = {};
            for (var i = 0; i < clients.length; i++) {
                var c = clients[i];
                var appId = c.resourceClass || c.resourceName || "unknown";
                if (!result[appId]) result[appId] = 0;
                result[appId]++;
            }
            JSON.stringify(result);
        "#;
        
        let reply = connection
            .call_method(
                Some("org.kde.KWin"),
                "/Scripting",
                Some("org.kde.kwin.Scripting"),
                "loadScript",
                &(script, "blazedock_count"),
            )
            .await;
        
        match reply {
            Ok(_) => debug!("KWin script loaded for window counting"),
            Err(e) => debug!("KWin script method failed: {}", e),
        }
        
        Ok(())
    }

    /// Start GNOME window tracking via D-Bus
    fn start_gnome_tracking(&self) {
        let tracker = self.clone();
        
        glib::spawn_future_local(async move {
            match tracker.poll_gnome_windows().await {
                Ok(_) => info!("GNOME window polling started"),
                Err(e) => warn!("Failed to start GNOME window tracking: {}", e),
            }
        });
        
        // Start periodic polling
        let tracker = self.clone();
        glib::timeout_add_seconds_local(2, move || {
            if !tracker.is_running() {
                return glib::ControlFlow::Break;
            }
            
            let tracker_clone = tracker.clone();
            glib::spawn_future_local(async move {
                if let Err(e) = tracker_clone.poll_gnome_windows().await {
                    debug!("GNOME window poll error: {}", e);
                }
            });
            
            glib::ControlFlow::Continue
        });
    }

    /// Poll GNOME windows via D-Bus
    async fn poll_gnome_windows(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let connection = zbus::Connection::session().await?;
        
        // Use org.gnome.Shell.Introspect for window information
        let reply = connection
            .call_method(
                Some("org.gnome.Shell.Introspect"),
                "/org/gnome/Shell/Introspect",
                Some("org.gnome.Shell.Introspect"),
                "GetWindows",
                &(),
            )
            .await;
        
        match reply {
            Ok(message) => {
                // Parse window list from the response
                self.parse_gnome_windows(&message)?;
            }
            Err(e) => {
                debug!("GNOME Shell.Introspect not available: {}", e);
                // Fall back to org.gnome.Shell.Extensions for window list
            }
        }
        
        Ok(())
    }

    /// Parse GNOME window response
    fn parse_gnome_windows(&self, message: &zbus::Message) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // GetWindows returns a{ta{sv}} - window ID -> properties dict
        // Simplified parsing - just extract basic info
        let body = message.body();
        
        // Try to deserialize as the expected format
        // If it fails, we'll just log and continue
        let result: Result<HashMap<u64, HashMap<String, zbus::zvariant::OwnedValue>>, _> = 
            body.deserialize();
        
        match result {
            Ok(window_map) => {
                let mut counts: HashMap<String, u32> = HashMap::new();
                let mut window_list = Vec::new();
                
                for (window_id, props) in window_map {
                    // Extract app-id using string conversion
                    let app_id = props.get("app-id")
                        .and_then(|v| {
                            // Try to get as string from the variant
                            let s: Result<String, _> = v.try_to_owned()
                                .ok()
                                .and_then(|ov| ov.try_into().ok())
                                .ok_or(());
                            s.ok()
                        })
                        .unwrap_or_else(|| "unknown".to_string());
                    
                    let title = props.get("title")
                        .and_then(|v| {
                            let s: Result<String, _> = v.try_to_owned()
                                .ok()
                                .and_then(|ov| ov.try_into().ok())
                                .ok_or(());
                            s.ok()
                        })
                        .unwrap_or_default();
                    
                    // Update counts
                    *counts.entry(app_id.clone()).or_insert(0) += 1;
                    
                    // Add to window list
                    window_list.push(WindowInfo {
                        id: window_id.to_string(),
                        title,
                        app_id,
                        is_active: false, // Simplified for now
                    });
                }
                
                let window_count = window_list.len();
                
                // Update internal state
                *self.app_window_counts.lock().unwrap() = counts;
                *self.windows.lock().unwrap() = window_list;
                
                debug!("GNOME: Found {} windows", window_count);
            }
            Err(e) => {
                debug!("Failed to parse GNOME window data: {}", e);
            }
        }
        
        Ok(())
    }

    /// Start Hyprland window tracking via IPC socket
    fn start_hyprland_tracking(&self) {
        let tracker = self.clone();
        
        // Initial poll
        glib::spawn_future_local(async move {
            if let Err(e) = tracker.poll_hyprland_windows().await {
                warn!("Hyprland window tracking failed: {}", e);
            }
        });
        
        // Periodic polling
        let tracker = self.clone();
        glib::timeout_add_seconds_local(2, move || {
            if !tracker.is_running() {
                return glib::ControlFlow::Break;
            }
            
            let tracker_clone = tracker.clone();
            glib::spawn_future_local(async move {
                if let Err(e) = tracker_clone.poll_hyprland_windows().await {
                    debug!("Hyprland poll error: {}", e);
                }
            });
            
            glib::ControlFlow::Continue
        });
    }

    /// Poll Hyprland windows via IPC
    async fn poll_hyprland_windows(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::UnixStream;
        
        let signature = std::env::var("HYPRLAND_INSTANCE_SIGNATURE")?;
        let socket_path = format!("/tmp/hypr/{}/.socket.sock", signature);
        
        let mut stream = UnixStream::connect(&socket_path).await?;
        stream.write_all(b"j/clients").await?;
        
        let mut response = String::new();
        stream.read_to_string(&mut response).await?;
        
        // Parse JSON response
        self.parse_hyprland_clients(&response)?;
        
        Ok(())
    }

    /// Parse Hyprland client list JSON
    fn parse_hyprland_clients(&self, json: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        #[derive(serde::Deserialize)]
        struct HyprClient {
            address: String,
            title: String,
            class: String,
            #[serde(default)]
            #[allow(dead_code)]
            workspace: i32,
        }
        
        let clients: Vec<HyprClient> = serde_json::from_str(json)?;
        
        let mut counts: HashMap<String, u32> = HashMap::new();
        let mut window_list = Vec::new();
        
        for client in clients {
            let app_id = client.class.clone();
            *counts.entry(app_id.clone()).or_insert(0) += 1;
            
            window_list.push(WindowInfo {
                id: client.address,
                title: client.title,
                app_id,
                is_active: false, // Would need active window query
            });
        }
        
        let window_count = window_list.len();
        
        *self.app_window_counts.lock().unwrap() = counts;
        *self.windows.lock().unwrap() = window_list;
        
        debug!("Hyprland: Found {} windows", window_count);
        Ok(())
    }

    /// Start Sway window tracking via IPC
    fn start_sway_tracking(&self) {
        let tracker = self.clone();
        
        // Initial poll
        glib::spawn_future_local(async move {
            if let Err(e) = tracker.poll_sway_windows().await {
                warn!("Sway window tracking failed: {}", e);
            }
        });
        
        // Periodic polling  
        let tracker = self.clone();
        glib::timeout_add_seconds_local(2, move || {
            if !tracker.is_running() {
                return glib::ControlFlow::Break;
            }
            
            let tracker_clone = tracker.clone();
            glib::spawn_future_local(async move {
                if let Err(e) = tracker_clone.poll_sway_windows().await {
                    debug!("Sway poll error: {}", e);
                }
            });
            
            glib::ControlFlow::Continue
        });
    }

    /// Poll Sway windows via IPC
    async fn poll_sway_windows(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::UnixStream;
        
        let socket_path = std::env::var("SWAYSOCK")?;
        let mut stream = UnixStream::connect(&socket_path).await?;
        
        // Sway IPC message format: magic | length | type | payload
        // Type 4 = get_tree
        let magic = b"i3-ipc";
        let msg_type: u32 = 4; // GET_TREE
        let payload: &[u8] = &[];
        
        stream.write_all(magic).await?;
        stream.write_all(&(payload.len() as u32).to_ne_bytes()).await?;
        stream.write_all(&msg_type.to_ne_bytes()).await?;
        stream.write_all(payload).await?;
        
        // Read response header
        let mut header = [0u8; 14]; // 6 magic + 4 len + 4 type
        stream.read_exact(&mut header).await?;
        
        let len = u32::from_ne_bytes([header[6], header[7], header[8], header[9]]) as usize;
        
        // Read response body
        let mut body = vec![0u8; len];
        stream.read_exact(&mut body).await?;
        
        let json = String::from_utf8(body)?;
        self.parse_sway_tree(&json)?;
        
        Ok(())
    }

    /// Parse Sway tree to extract windows
    fn parse_sway_tree(&self, json: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        #[derive(serde::Deserialize)]
        struct SwayNode {
            #[serde(default)]
            app_id: Option<String>,
            #[serde(default)]
            name: Option<String>,
            #[serde(default)]
            nodes: Vec<SwayNode>,
            #[serde(default)]
            floating_nodes: Vec<SwayNode>,
            #[serde(default)]
            focused: bool,
            #[serde(default)]
            id: i64,
            #[serde(default)]
            #[serde(rename = "type")]
            node_type: Option<String>,
        }
        
        fn collect_windows(node: &SwayNode, windows: &mut Vec<WindowInfo>, counts: &mut HashMap<String, u32>) {
            // Check if this is a window (con with app_id)
            if node.node_type.as_deref() == Some("con") {
                if let Some(app_id) = &node.app_id {
                    *counts.entry(app_id.clone()).or_insert(0) += 1;
                    windows.push(WindowInfo {
                        id: node.id.to_string(),
                        title: node.name.clone().unwrap_or_default(),
                        app_id: app_id.clone(),
                        is_active: node.focused,
                    });
                }
            }
            
            // Recurse into children
            for child in &node.nodes {
                collect_windows(child, windows, counts);
            }
            for child in &node.floating_nodes {
                collect_windows(child, windows, counts);
            }
        }
        
        let root: SwayNode = serde_json::from_str(json)?;
        let mut windows = Vec::new();
        let mut counts = HashMap::new();
        
        collect_windows(&root, &mut windows, &mut counts);
        
        *self.app_window_counts.lock().unwrap() = counts;
        *self.windows.lock().unwrap() = windows.clone();
        
        debug!("Sway: Found {} windows", windows.len());
        Ok(())
    }

    /// Get number of windows for a specific app_id
    pub fn get_window_count(&self, app_id: &str) -> u32 {
        let counts = self.app_window_counts.lock().unwrap();
        
        // Try exact match first
        if let Some(count) = counts.get(app_id) {
            return *count;
        }
        
        // Try case-insensitive match
        let app_id_lower = app_id.to_lowercase();
        for (key, count) in counts.iter() {
            if key.to_lowercase() == app_id_lower || 
               key.to_lowercase().contains(&app_id_lower) ||
               app_id_lower.contains(&key.to_lowercase()) {
                return *count;
            }
        }
        
        0
    }

    /// Get all windows for a specific app_id
    pub fn get_windows_for_app(&self, app_id: &str) -> Vec<WindowInfo> {
        let windows = self.windows.lock().unwrap();
        let app_id_lower = app_id.to_lowercase();
        
        windows.iter()
            .filter(|w| {
                w.app_id.to_lowercase() == app_id_lower ||
                w.app_id.to_lowercase().contains(&app_id_lower) ||
                app_id_lower.contains(&w.app_id.to_lowercase())
            })
            .cloned()
            .collect()
    }

    /// Get all tracked windows
    pub fn get_all_windows(&self) -> Vec<WindowInfo> {
        self.windows.lock().unwrap().clone()
    }

    /// Update window count for an app (can be called from external process)
    pub fn set_window_count(&self, app_id: &str, count: u32) {
        let mut counts = self.app_window_counts.lock().unwrap();
        if count == 0 {
            counts.remove(app_id);
        } else {
            counts.insert(app_id.to_string(), count);
        }
    }

    /// Get the detected desktop environment
    pub fn get_desktop_environment(&self) -> DesktopEnvironment {
        *self.desktop.lock().unwrap()
    }

    /// Check if tracker is running
    pub fn is_running(&self) -> bool {
        *self.running.lock().unwrap()
    }

    /// Stop tracking windows
    pub fn stop(&self) {
        let mut running = self.running.lock().unwrap();
        *running = false;
        info!("Window tracker stopped");
    }
}

impl Default for WindowTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_desktop_detection() {
        // Test that detection doesn't panic
        let _desktop = WindowTracker::detect_desktop_environment();
    }

    #[test]
    fn test_window_count_operations() {
        let tracker = WindowTracker::new();
        
        // Initially zero
        assert_eq!(tracker.get_window_count("firefox"), 0);
        
        // Set count
        tracker.set_window_count("firefox", 3);
        assert_eq!(tracker.get_window_count("firefox"), 3);
        
        // Clear count
        tracker.set_window_count("firefox", 0);
        assert_eq!(tracker.get_window_count("firefox"), 0);
    }

    #[test]
    fn test_case_insensitive_matching() {
        let tracker = WindowTracker::new();
        
        tracker.set_window_count("Firefox", 2);
        assert_eq!(tracker.get_window_count("firefox"), 2);
        assert_eq!(tracker.get_window_count("FIREFOX"), 2);
    }
}
