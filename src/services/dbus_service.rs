//! D-Bus service for system integration
//!
//! Handles notification listening and Unity LauncherEntry badges.

use zbus::{connection, proxy};
use log::{info, error, debug};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

/// Event types for D-Bus integration
#[derive(Debug, Clone)]
pub enum DBusEvent {
    /// Badge update (app_id, count, visible)
    BadgeUpdate(String, u32, bool),
}

#[proxy(
    interface = "org.freedesktop.Notifications",
    default_service = "org.freedesktop.Notifications",
    default_path = "/org/freedesktop/Notifications"
)]
trait Notifications {
    #[zbus(signal)]
    fn action_invoked(&self, id: u32, action_key: &str) -> zbus::Result<()>;

    #[zbus(signal)]
    fn notification_closed(&self, id: u32, reason: u32) -> zbus::Result<()>;
}

/// D-Bus service for BlazeDock
pub struct DBusService {
    event_tx: broadcast::Sender<DBusEvent>,
    running: Arc<Mutex<bool>>,
}

impl DBusService {
    /// Create a new D-Bus service
    pub fn new() -> (Self, broadcast::Receiver<DBusEvent>) {
        let (tx, rx) = broadcast::channel(100);
        (
            Self {
                event_tx: tx,
                running: Arc::new(Mutex::new(false)),
            },
            rx,
        )
    }

    /// Start the D-Bus service
    pub fn start(&self) {
        let mut running = self.running.lock().unwrap();
        if *running {
            return;
        }
        *running = true;
        
        let tx = self.event_tx.clone();
        let running_flag = Arc::clone(&self.running);

        tokio::spawn(async move {
            info!("D-Bus service starting...");
            
            let conn = match connection::Builder::session().unwrap().build().await {
                Ok(c) => c,
                Err(e) => {
                    error!("Failed to connect to D-Bus session bus: {}", e);
                    return;
                }
            };

            // 1. Listen for Unity LauncherEntry (Standard for badges)
            // Implementation placeholder - Unity requires complex property monitoring
            debug!("Unity LauncherEntry listener placeholder");

            // 2. Simple Notification Counter (Fallback)
            debug!("D-Bus service fully initialized");
            
            while *running_flag.lock().unwrap() {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        });
    }

    /// Stop the D-Bus service
    pub fn stop(&self) {
        let mut running = self.running.lock().unwrap();
        *running = false;
    }
}
