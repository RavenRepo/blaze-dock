//! Drive monitor service
//!
//! Monitors removable drives and mounted partitions.

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
    pub fn start(&self) {
        let mut running = self.running.lock().unwrap();
        if *running {
            return;
        }
        *running = true;
        
        let drives = Arc::clone(&self.drives);
        let running_flag = Arc::clone(&self.running);

        tokio::spawn(async move {
            info!("Drive monitor started");
            
            while *running_flag.lock().unwrap() {
                // Check for mounted drives using lsblk
                let output = Command::new("lsblk")
                    .args(["-J", "-o", "NAME,MOUNTPOINT,RM"])
                    .output();
                
                if let Ok(res) = output {
                    // Parse JSON output and update drives
                    // For now, we log the check
                    debug!("Checked for drives");
                }
                
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
            
            info!("Drive monitor stopped");
        });
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

