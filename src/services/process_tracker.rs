//! Process tracker service
//!
//! Tracks running applications by checking process names.
//! This is a temporary solution until proper window tracking is implemented.

use log::{debug, info};
use std::collections::HashMap;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Maps app commands to their process names
fn command_to_process_name(command: &str) -> String {
    // Extract the base command name
    command.split_whitespace().next().unwrap_or(command).to_string()
}

/// Update the running state of all apps in one pass
fn update_all_apps(apps: &Arc<Mutex<HashMap<String, bool>>>) {
    // Get all running processes in one command
    let output = Command::new("ps")
        .args(["-e", "-o", "comm="])
        .output();
    
    let running_processes: std::collections::HashSet<String> = match output {
        Ok(res) => String::from_utf8_lossy(&res.stdout)
            .lines()
            .map(|s| s.trim().to_string())
            .collect(),
        Err(_) => return,
    };

    let mut apps_guard = apps.lock().unwrap();
    for (app_name, running) in apps_guard.iter_mut() {
        let is_running = running_processes.contains(app_name);
        if *running != is_running {
            debug!("App '{}' running state changed: {}", app_name, is_running);
            *running = is_running;
        }
    }
}

/// Process tracker for monitoring running applications
#[derive(Clone)]
pub struct ProcessTracker {
    apps: Arc<Mutex<HashMap<String, bool>>>,
    running: Arc<Mutex<bool>>,
}

impl ProcessTracker {
    /// Create a new process tracker
    pub fn new() -> Self {
        Self {
            apps: Arc::new(Mutex::new(HashMap::new())),
            running: Arc::new(Mutex::new(false)),
        }
    }

    /// Register an application to track
    pub fn register_app(&self, command: &str) {
        let process_name = command_to_process_name(command);
        let mut apps = self.apps.lock().unwrap();
        apps.insert(process_name.clone(), false);
        debug!("Registered app for tracking: {}", process_name);
    }

    /// Check if an app is currently running
    pub fn is_running(&self, command: &str) -> bool {
        let process_name = command_to_process_name(command);
        let apps = self.apps.lock().unwrap();
        apps.get(&process_name).copied().unwrap_or(false)
    }

    /// Start tracking processes
    pub fn start(&self) {
        let mut running = self.running.lock().unwrap();
        if *running {
            return; // Already running
        }
        *running = true;
        drop(running);

        let apps = Arc::clone(&self.apps);
        let running_flag = Arc::clone(&self.running);

        thread::spawn(move || {
            info!("Process tracker started");
            
            loop {
                // Check if we should stop
                {
                    let running = running_flag.lock().unwrap();
                    if !*running {
                        break;
                    }
                }

                update_all_apps(&apps);

                // Check every 2 seconds
                thread::sleep(Duration::from_secs(2));
            }

            info!("Process tracker stopped");
        });
    }

    /// Stop tracking processes
    pub fn stop(&self) {
        let mut running = self.running.lock().unwrap();
        *running = false;
        info!("Process tracker stop requested");
    }
}

impl Default for ProcessTracker {
    fn default() -> Self {
        Self::new()
    }
}

