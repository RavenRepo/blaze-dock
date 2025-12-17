//! Multi-monitor support for BlazeDock
//!
//! Manages dock instances across multiple displays.

use gtk::prelude::*;
use gtk::glib;
use gtk::gdk;
use log::{info, debug, warn};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Monitor information
#[derive(Debug, Clone)]
pub struct MonitorInfo {
    pub id: String,
    pub name: String,
    pub geometry: gdk::Rectangle,
    pub scale_factor: i32,
    pub is_primary: bool,
    pub connector: String,
}

/// Multi-monitor mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MultiMonitorMode {
    /// Dock only on primary monitor
    PrimaryOnly,
    /// Dock on all monitors (cloned)
    AllMonitors,
    /// Dock follows mouse
    FollowMouse,
    /// Different dock per monitor (configured)
    PerMonitor,
}

/// Multi-monitor service
#[derive(Clone)]
pub struct MultiMonitorService {
    monitors: Arc<Mutex<HashMap<String, MonitorInfo>>>,
    mode: Arc<Mutex<MultiMonitorMode>>,
    primary_monitor: Arc<Mutex<Option<String>>>,
    current_monitor: Arc<Mutex<Option<String>>>,
    on_monitor_change: Arc<Mutex<Vec<Box<dyn Fn(&MonitorInfo) + Send + Sync>>>>,
}

impl MultiMonitorService {
    /// Create a new multi-monitor service
    pub fn new() -> Self {
        let service = Self {
            monitors: Arc::new(Mutex::new(HashMap::new())),
            mode: Arc::new(Mutex::new(MultiMonitorMode::PrimaryOnly)),
            primary_monitor: Arc::new(Mutex::new(None)),
            current_monitor: Arc::new(Mutex::new(None)),
            on_monitor_change: Arc::new(Mutex::new(Vec::new())),
        };

        service.scan_monitors();
        service
    }

    /// Scan for connected monitors
    pub fn scan_monitors(&self) {
        let display = match gdk::Display::default() {
            Some(d) => d,
            None => {
                warn!("No display available");
                return;
            }
        };

        let mut monitors = self.monitors.lock().unwrap();
        monitors.clear();

        let monitor_list = display.monitors();
        let n_monitors = monitor_list.n_items();
        
        info!("Found {} monitors", n_monitors);

        for i in 0..n_monitors {
            if let Some(monitor) = monitor_list.item(i).and_downcast::<gdk::Monitor>() {
                let geometry = monitor.geometry();
                let connector = monitor.connector().map(|s| s.to_string()).unwrap_or_default();
                let id = format!("monitor-{}", i);
                
                let info = MonitorInfo {
                    id: id.clone(),
                    name: monitor.model().map(|s| s.to_string()).unwrap_or_else(|| format!("Monitor {}", i)),
                    geometry,
                    scale_factor: monitor.scale_factor(),
                    is_primary: i == 0, // First monitor is typically primary
                    connector,
                };

                debug!("Monitor {}: {} ({}x{} at {},{})", 
                    i, info.name, 
                    geometry.width(), geometry.height(),
                    geometry.x(), geometry.y()
                );

                if info.is_primary {
                    *self.primary_monitor.lock().unwrap() = Some(id.clone());
                }

                monitors.insert(id, info);
            }
        }

        // Set current monitor to primary if not set
        if self.current_monitor.lock().unwrap().is_none() {
            *self.current_monitor.lock().unwrap() = self.primary_monitor.lock().unwrap().clone();
        }
    }

    /// Start monitoring for display changes
    pub fn start_monitoring(&self) {
        let service = self.clone();
        
        if let Some(display) = gdk::Display::default() {
            // Monitor for display changes
            display.connect_opened(move |_| {
                debug!("Display connection opened");
                service.scan_monitors();
            });
        }

        // Periodic rescan (handles hotplug)
        let service_clone = self.clone();
        glib::timeout_add_seconds_local(5, move || {
            service_clone.check_for_changes();
            glib::ControlFlow::Continue
        });
    }

    /// Check for monitor changes
    fn check_for_changes(&self) {
        let old_count = self.monitors.lock().unwrap().len();
        self.scan_monitors();
        let new_count = self.monitors.lock().unwrap().len();
        
        if old_count != new_count {
            info!("Monitor configuration changed: {} -> {} monitors", old_count, new_count);
            self.notify_change();
        }
    }

    /// Notify callbacks of monitor change
    fn notify_change(&self) {
        let current_id = self.current_monitor.lock().unwrap().clone();
        if let Some(id) = current_id {
            if let Some(monitor) = self.get_monitor(&id) {
                let callbacks = self.on_monitor_change.lock().unwrap();
                for callback in callbacks.iter() {
                    callback(&monitor);
                }
            }
        }
    }

    /// Register callback for monitor changes
    pub fn on_monitor_change<F>(&self, callback: F)
    where
        F: Fn(&MonitorInfo) + Send + Sync + 'static,
    {
        let mut callbacks = self.on_monitor_change.lock().unwrap();
        callbacks.push(Box::new(callback));
    }

    /// Set multi-monitor mode
    pub fn set_mode(&self, mode: MultiMonitorMode) {
        *self.mode.lock().unwrap() = mode;
        info!("Multi-monitor mode set to: {:?}", mode);
    }

    /// Get current mode
    pub fn get_mode(&self) -> MultiMonitorMode {
        *self.mode.lock().unwrap()
    }

    /// Get all monitors
    pub fn get_monitors(&self) -> Vec<MonitorInfo> {
        self.monitors.lock().unwrap().values().cloned().collect()
    }

    /// Get monitor by ID
    pub fn get_monitor(&self, id: &str) -> Option<MonitorInfo> {
        self.monitors.lock().unwrap().get(id).cloned()
    }

    /// Get primary monitor
    pub fn get_primary_monitor(&self) -> Option<MonitorInfo> {
        let id = self.primary_monitor.lock().unwrap().clone()?;
        self.get_monitor(&id)
    }

    /// Get current active monitor
    pub fn get_current_monitor(&self) -> Option<MonitorInfo> {
        let id = self.current_monitor.lock().unwrap().clone()?;
        self.get_monitor(&id)
    }

    /// Set current monitor by ID
    pub fn set_current_monitor(&self, id: &str) {
        if self.monitors.lock().unwrap().contains_key(id) {
            *self.current_monitor.lock().unwrap() = Some(id.to_string());
            debug!("Current monitor set to: {}", id);
        }
    }

    /// Find monitor containing point
    pub fn monitor_at_point(&self, x: i32, y: i32) -> Option<MonitorInfo> {
        let monitors = self.monitors.lock().unwrap();
        
        for monitor in monitors.values() {
            let geom = &monitor.geometry;
            if x >= geom.x() && x < geom.x() + geom.width() &&
               y >= geom.y() && y < geom.y() + geom.height() {
                return Some(monitor.clone());
            }
        }
        
        None
    }

    /// Get monitor for a position based on current mode
    pub fn get_target_monitor(&self) -> Option<MonitorInfo> {
        match *self.mode.lock().unwrap() {
            MultiMonitorMode::PrimaryOnly => self.get_primary_monitor(),
            MultiMonitorMode::AllMonitors => self.get_primary_monitor(), // Return primary, dock will be cloned
            MultiMonitorMode::FollowMouse => {
                // Get mouse position (simplified - would need actual pointer tracking)
                self.get_current_monitor()
            }
            MultiMonitorMode::PerMonitor => self.get_current_monitor(),
        }
    }

    /// Get dock position for a specific monitor
    pub fn get_dock_geometry(&self, monitor: &MonitorInfo, dock_width: i32, dock_height: i32, margin: i32) -> (i32, i32) {
        let geom = &monitor.geometry;
        
        // Center horizontally at bottom
        let x = geom.x() + (geom.width() - dock_width) / 2;
        let y = geom.y() + geom.height() - dock_height - margin;
        
        (x, y)
    }

    /// Check if running in multi-monitor setup
    pub fn is_multi_monitor(&self) -> bool {
        self.monitors.lock().unwrap().len() > 1
    }

    /// Get number of monitors
    pub fn monitor_count(&self) -> usize {
        self.monitors.lock().unwrap().len()
    }
}

impl Default for MultiMonitorService {
    fn default() -> Self {
        Self::new()
    }
}

