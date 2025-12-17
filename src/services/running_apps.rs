//! Running applications service
//!
//! Discovers and tracks running GUI applications that should appear in the dock.

use log::{debug, info};
use std::collections::{HashMap, HashSet};
use std::process::Command;
use std::sync::{Arc, Mutex};

use crate::utils::desktop_entry::{DesktopEntry, APP_DIRS};

/// Information about a running application
#[derive(Debug, Clone)]
pub struct RunningApp {
    pub name: String,
    pub icon: String,
    pub command: String,
    pub desktop_file: Option<String>,
    pub process_name: String,
}

/// Service for tracking running GUI applications
pub struct RunningAppsService {
    /// Cache of desktop entries indexed by process name
    desktop_cache: Arc<Mutex<HashMap<String, DesktopEntry>>>,
    /// Currently running apps (not in pinned list)
    running_apps: Arc<Mutex<Vec<RunningApp>>>,
}

impl RunningAppsService {
    /// Create a new running apps service
    pub fn new() -> Self {
        let service = Self {
            desktop_cache: Arc::new(Mutex::new(HashMap::new())),
            running_apps: Arc::new(Mutex::new(Vec::new())),
        };
        service.build_cache();
        service
    }

    /// Build cache of desktop entries indexed by process name
    fn build_cache(&self) {
        let mut cache = self.desktop_cache.lock().unwrap();
        
        for dir in APP_DIRS {
            if let Ok(read_dir) = std::fs::read_dir(dir) {
                for entry in read_dir.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.extension().map(|e| e == "desktop").unwrap_or(false) {
                        if let Ok(desktop) = DesktopEntry::parse(&path) {
                            if desktop.is_visible_app() {
                                if let Some(exec) = &desktop.exec {
                                    // Extract process name from exec
                                    let process_name = exec
                                        .split_whitespace()
                                        .next()
                                        .unwrap_or("")
                                        .split('/')
                                        .last()
                                        .unwrap_or("")
                                        .to_string();
                                    
                                    if !process_name.is_empty() {
                                        cache.insert(process_name.to_lowercase(), desktop);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        info!("Desktop entry cache built: {} entries", cache.len());
    }

    /// Get list of running GUI applications
    pub fn get_running_apps(&self, pinned_commands: &[String]) -> Vec<RunningApp> {
        // Get all running processes
        let output = Command::new("ps")
            .args(["-e", "-o", "comm="])
            .output();
        
        let running_processes: HashSet<String> = match output {
            Ok(res) => String::from_utf8_lossy(&res.stdout)
                .lines()
                .map(|s| s.trim().to_lowercase())
                .collect(),
            Err(_) => return Vec::new(),
        };

        // Convert pinned commands to process names for comparison
        let pinned_process_names: HashSet<String> = pinned_commands
            .iter()
            .map(|cmd| {
                cmd.split_whitespace()
                    .next()
                    .unwrap_or("")
                    .split('/')
                    .last()
                    .unwrap_or("")
                    .to_lowercase()
            })
            .collect();

        let cache = self.desktop_cache.lock().unwrap();
        let mut apps = Vec::new();

        for (process_name, desktop) in cache.iter() {
            // Check if this process is running
            if running_processes.contains(process_name) {
                // Skip if it's already pinned
                if pinned_process_names.contains(process_name) {
                    continue;
                }

                let app = RunningApp {
                    name: desktop.name.clone().unwrap_or_else(|| process_name.clone()),
                    icon: desktop.icon.clone().unwrap_or_else(|| "application-x-executable".to_string()),
                    command: desktop.exec_command().unwrap_or_else(|| process_name.clone()),
                    desktop_file: Some(desktop.path.to_string_lossy().to_string()),
                    process_name: process_name.clone(),
                };
                
                debug!("Found running app: {} ({})", app.name, process_name);
                apps.push(app);
            }
        }

        // Sort alphabetically
        apps.sort_by(|a, b| a.name.cmp(&b.name));
        apps
    }
}

impl Default for RunningAppsService {
    fn default() -> Self {
        Self::new()
    }
}

