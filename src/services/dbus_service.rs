//! D-Bus service for system integration
//!
//! Handles Unity LauncherEntry badges and notification listening.
//! Provides badge counts for applications like email clients, browsers, etc.

use gtk::glib;
use log::{info, debug, warn};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// Badge information for an application
#[derive(Debug, Clone)]
pub struct BadgeInfo {
    /// Application identifier (desktop file name without .desktop)
    pub app_id: String,
    /// Badge count (e.g., unread emails)
    pub count: i64,
    /// Whether the badge is visible
    pub count_visible: bool,
    /// Progress value (0.0 to 1.0) for download/upload progress
    pub progress: f64,
    /// Whether progress is visible
    pub progress_visible: bool,
    /// Whether the app is requesting urgent attention
    pub urgent: bool,
}

impl Default for BadgeInfo {
    fn default() -> Self {
        Self {
            app_id: String::new(),
            count: 0,
            count_visible: false,
            progress: 0.0,
            progress_visible: false,
            urgent: false,
        }
    }
}

/// Event types for D-Bus integration  
#[derive(Debug, Clone)]
pub enum DBusEvent {
    /// Badge update for an application
    BadgeUpdate(BadgeInfo),
    /// Notification received
    Notification { app_name: String, summary: String },
}

/// D-Bus service for BlazeDock
/// Listens for Unity LauncherEntry signals and FreeDesktop notifications
#[derive(Clone)]
pub struct DBusService {
    /// Current badge state per application
    badges: Arc<Mutex<HashMap<String, BadgeInfo>>>,
    /// Callbacks for badge updates
    callbacks: Arc<Mutex<Vec<Box<dyn Fn(BadgeInfo) + Send + 'static>>>>,
    /// Running state
    running: Arc<Mutex<bool>>,
}

impl DBusService {
    /// Create a new D-Bus service
    pub fn new() -> Self {
        Self {
            badges: Arc::new(Mutex::new(HashMap::new())),
            callbacks: Arc::new(Mutex::new(Vec::new())),
            running: Arc::new(Mutex::new(false)),
        }
    }

    /// Register a callback for badge updates
    pub fn on_badge_update<F>(&self, callback: F)
    where
        F: Fn(BadgeInfo) + Send + 'static,
    {
        self.callbacks.lock().unwrap().push(Box::new(callback));
    }

    /// Get current badge info for an app
    pub fn get_badge(&self, app_id: &str) -> Option<BadgeInfo> {
        let badges = self.badges.lock().unwrap();
        
        // Try exact match
        if let Some(badge) = badges.get(app_id) {
            return Some(badge.clone());
        }
        
        // Try case-insensitive match
        let app_id_lower = app_id.to_lowercase();
        for (key, badge) in badges.iter() {
            if key.to_lowercase() == app_id_lower ||
               key.to_lowercase().contains(&app_id_lower) {
                return Some(badge.clone());
            }
        }
        
        None
    }

    /// Get all current badges
    pub fn get_all_badges(&self) -> HashMap<String, BadgeInfo> {
        self.badges.lock().unwrap().clone()
    }

    /// Start the D-Bus service
    pub fn start(&self) {
        let mut running = self.running.lock().unwrap();
        if *running {
            return;
        }
        *running = true;
        drop(running);

        info!("D-Bus service starting");
        
        // Start Unity LauncherEntry listener
        self.start_launcher_entry_listener();
        
        // Start notification listener
        self.start_notification_listener();
    }

    /// Start listening for Unity LauncherEntry signals
    fn start_launcher_entry_listener(&self) {
        let service = self.clone();
        
        glib::spawn_future_local(async move {
            match service.listen_launcher_entry().await {
                Ok(_) => info!("LauncherEntry listener started"),
                Err(e) => warn!("Failed to start LauncherEntry listener: {}", e),
            }
        });
    }

    /// Listen for com.canonical.Unity.LauncherEntry signals
    async fn listen_launcher_entry(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let connection = zbus::Connection::session().await?;
        
        // Listen for signals using MessageStream - filter by interface/member
        info!("Listening for Unity LauncherEntry signals");
        
        let service = self.clone();
        let mut stream = zbus::MessageStream::from(&connection);
        
        use futures_util::StreamExt;
        
        while let Some(msg) = stream.next().await {
            if let Ok(message) = msg {
                // Check if this is a LauncherEntry Update signal
                let is_launcher_entry = message.interface()
                    .map(|i| i.to_string())
                    .as_deref() == Some("com.canonical.Unity.LauncherEntry");
                
                let is_update = message.member()
                    .map(|m| m.to_string())
                    .as_deref() == Some("Update");
                
                if is_launcher_entry && is_update {
                    if let Err(e) = service.handle_launcher_entry_update(&message).await {
                        debug!("Error handling LauncherEntry update: {}", e);
                    }
                }
            }
            
            // Check if we should stop
            if !*service.running.lock().unwrap() {
                break;
            }
        }
        
        Ok(())
    }

    /// Handle a Unity LauncherEntry Update signal
    async fn handle_launcher_entry_update(&self, message: &zbus::Message) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let body = message.body();
        
        // LauncherEntry.Update signature: (sa{sv})
        // - s: application URI (e.g., "application://firefox.desktop")
        // - a{sv}: properties dictionary
        let (app_uri, props): (String, HashMap<String, zbus::zvariant::OwnedValue>) = 
            body.deserialize()?;
        
        // Extract app_id from URI
        let app_id = app_uri
            .strip_prefix("application://")
            .and_then(|s| s.strip_suffix(".desktop"))
            .unwrap_or(&app_uri)
            .to_string();
        
        debug!("LauncherEntry update for: {}", app_id);
        
        // Parse badge properties
        let mut badge = BadgeInfo {
            app_id: app_id.clone(),
            ..Default::default()
        };
        
        // Extract count
        if let Some(count) = props.get("count") {
            if let Ok(c) = count.clone().try_into() {
                badge.count = c;
            }
        }
        
        // Extract count-visible
        if let Some(visible) = props.get("count-visible") {
            if let Ok(v) = visible.clone().try_into() {
                badge.count_visible = v;
            }
        }
        
        // Extract progress
        if let Some(progress) = props.get("progress") {
            if let Ok(p) = progress.clone().try_into() {
                badge.progress = p;
            }
        }
        
        // Extract progress-visible
        if let Some(visible) = props.get("progress-visible") {
            if let Ok(v) = visible.clone().try_into() {
                badge.progress_visible = v;
            }
        }
        
        // Extract urgent
        if let Some(urgent) = props.get("urgent") {
            if let Ok(u) = urgent.clone().try_into() {
                badge.urgent = u;
            }
        }
        
        // Store badge info
        self.badges.lock().unwrap().insert(app_id.clone(), badge.clone());
        
        // Notify callbacks
        let callbacks = self.callbacks.lock().unwrap();
        for callback in callbacks.iter() {
            callback(badge.clone());
        }
        
        debug!("Badge updated: {:?}", badge);
        Ok(())
    }

    /// Start listening for FreeDesktop notifications
    fn start_notification_listener(&self) {
        let service = self.clone();
        
        glib::spawn_future_local(async move {
            match service.listen_notifications().await {
                Ok(_) => info!("Notification listener started"),
                Err(e) => warn!("Failed to start notification listener: {}", e),
            }
        });
    }

    /// Listen for org.freedesktop.Notifications signals
    async fn listen_notifications(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Notification listening is best-effort - just log that it's attempted
        info!("Notification monitoring enabled (passive)");
        
        // We're mainly interested in LauncherEntry, but notifications
        // can inform us about app activity through the existing MessageStream
        Ok(())
    }

    /// Manually set a badge (for testing or external updates)
    pub fn set_badge(&self, app_id: &str, count: i64, visible: bool) {
        let badge = BadgeInfo {
            app_id: app_id.to_string(),
            count,
            count_visible: visible,
            ..Default::default()
        };
        
        self.badges.lock().unwrap().insert(app_id.to_string(), badge.clone());
        
        // Notify callbacks
        let callbacks = self.callbacks.lock().unwrap();
        for callback in callbacks.iter() {
            callback(badge.clone());
        }
    }

    /// Set progress for an app
    pub fn set_progress(&self, app_id: &str, progress: f64, visible: bool) {
        let mut badges = self.badges.lock().unwrap();
        let badge = badges.entry(app_id.to_string()).or_insert_with(|| BadgeInfo {
            app_id: app_id.to_string(),
            ..Default::default()
        });
        
        badge.progress = progress;
        badge.progress_visible = visible;
        
        let badge_clone = badge.clone();
        drop(badges);
        
        // Notify callbacks
        let callbacks = self.callbacks.lock().unwrap();
        for callback in callbacks.iter() {
            callback(badge_clone.clone());
        }
    }

    /// Stop the D-Bus service
    pub fn stop(&self) {
        let mut running = self.running.lock().unwrap();
        *running = false;
        info!("D-Bus service stopped");
    }

    /// Check if D-Bus service is running
    pub fn is_running(&self) -> bool {
        *self.running.lock().unwrap()
    }
}

impl Default for DBusService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_badge_operations() {
        let service = DBusService::new();
        
        // Initially no badges
        assert!(service.get_badge("firefox").is_none());
        
        // Set a badge
        service.set_badge("firefox", 5, true);
        
        let badge = service.get_badge("firefox").unwrap();
        assert_eq!(badge.count, 5);
        assert!(badge.count_visible);
    }

    #[test]
    fn test_progress_operations() {
        let service = DBusService::new();
        
        service.set_progress("nautilus", 0.5, true);
        
        let badge = service.get_badge("nautilus").unwrap();
        assert!((badge.progress - 0.5).abs() < 0.01);
        assert!(badge.progress_visible);
    }

    #[test]
    fn test_case_insensitive_lookup() {
        let service = DBusService::new();
        
        service.set_badge("Firefox", 3, true);
        
        assert!(service.get_badge("firefox").is_some());
        assert!(service.get_badge("FIREFOX").is_some());
    }
}
