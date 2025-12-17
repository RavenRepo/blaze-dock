//! Drive monitor service
//!
//! Monitors removable drives and mounted partitions.
//! Note: Full async implementation requires proper runtime setup.

use log::{info, debug};
use std::sync::{Arc, Mutex};
use std::process::Command;

/// Drive information
#[derive(Debug, Clone)]
pub struct DriveInfo {
    pub name: String,
    pub mount_point: String,
    pub is_removable: bool,
}

/// Drive monitor for tracking removable media
/// Currently a placeholder - full implementation pending async runtime setup
pub struct DriveMonitor {
    drives: Arc<Mutex<Vec<DriveInfo>>>,
    running: Arc<Mutex<bool>>,
}

impl DriveMonitor {
    /// Create a new drive monitor
    pub fn new() -> Self {
        Self {
            drives: Arc::new(Mutex::new(Vec::new())),
            running: Arc::new(Mutex::new(false)),
        }
    }

    /// Start monitoring drives
    /// Note: Currently a no-op placeholder. Full drive monitoring will be
    /// implemented using GIO/udev or periodic lsblk polling.
    pub fn start(&self) {
        let mut running = self.running.lock().unwrap();
        if *running {
            return;
        }
        *running = true;
        
        info!("Drive monitor initialized (placeholder mode)");
        debug!("Full drive monitoring pending async runtime setup");
        
        // TODO: Implement proper drive monitoring using:
        // - GIO volume monitor
        // - udev events
        // - Periodic lsblk polling via glib::timeout_add
        // For now, this is a safe placeholder that doesn't crash.
    }

    /// Get list of currently mounted drives
    pub fn get_drives(&self) -> Vec<DriveInfo> {
        // Do a one-time sync check for drives
        let output = Command::new("lsblk")
            .args(["-J", "-o", "NAME,MOUNTPOINT,RM"])
            .output();
        
        if let Ok(res) = output {
            if res.status.success() {
                // For now, just return the cached list
                // Full parsing will be implemented later
                debug!("Drive check completed");
            }
        }
        
        self.drives.lock().unwrap().clone()
    }

    /// Check if monitor is running
    pub fn is_running(&self) -> bool {
        *self.running.lock().unwrap()
    }

    /// Stop monitoring drives
    pub fn stop(&self) {
        let mut running = self.running.lock().unwrap();
        *running = false;
    }
}

impl Default for DriveMonitor {
    fn default() -> Self {
        Self::new()
    }
}
