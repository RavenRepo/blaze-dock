//! Dock window implementation
//!
//! Creates and manages the main dock window with Wayland Layer Shell integration.

use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box, Orientation};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use log::{debug, info, warn};

use crate::config::{DockPosition, Settings};
use crate::services::ProcessTracker;
use crate::ui::{DockItem, RunningState, MagnificationController};
use std::cell::RefCell;
use std::rc::Rc;

/// Main dock window wrapper
pub struct DockWindow {
    window: ApplicationWindow,
    dock_items: Rc<RefCell<Vec<(String, Rc<RefCell<DockItem>>)>>>,
    process_tracker: ProcessTracker,
    magnification: Rc<RefCell<MagnificationController>>,
}

impl DockWindow {
    /// Create a new dock window
    pub fn new(app: &Application, settings: &Settings) -> Self {
        // Check if we should use layer shell
        // Currently disabled by default due to KDE Plasma 6 compatibility issues
        // Set BLAZEDOCK_LAYER_SHELL=1 to force enable on compatible compositors (Sway, Hyprland)
        let force_layer_shell = std::env::var("BLAZEDOCK_LAYER_SHELL").is_ok();
        let use_layer_shell = force_layer_shell && gtk4_layer_shell::is_supported();
        
        if force_layer_shell && !gtk4_layer_shell::is_supported() {
            warn!("Layer shell requested but not supported on this compositor");
        } else if use_layer_shell {
            info!("Layer shell enabled (via BLAZEDOCK_LAYER_SHELL)");
        } else {
            info!("Running in floating window mode (layer shell disabled)");
            info!("Tip: Set BLAZEDOCK_LAYER_SHELL=1 for Sway/Hyprland");
        }

        // Create the window
        let window = ApplicationWindow::builder()
            .application(app)
            .title("BlazeDock")
            .decorated(false)
            .resizable(true)
            .build();

        // Configure based on mode
        if use_layer_shell {
            Self::setup_layer_shell(&window, settings);
        } else {
            // Floating window mode - position on left edge
            Self::setup_floating_window(&window, settings);
        }

        // Add CSS class for styling
        window.add_css_class("blazedock-window");

        // Store dock items reference
        let dock_items = Rc::new(RefCell::new(Vec::new()));

        // Create and add dock content
        let magnification = Rc::new(RefCell::new(MagnificationController::new(
            settings.hover_zoom_scale,
            2,
        )));
        let dock_content = Self::create_dock_content(settings, &dock_items, &magnification);
        
        // Set size based on position
        let (width, height) = match settings.position {
            DockPosition::Left | DockPosition::Right => {
                (settings.dock_size as i32, 500)
            }
            DockPosition::Top | DockPosition::Bottom => {
                (800, settings.dock_size as i32)
            }
        };
        
        dock_content.set_size_request(width, height);
        window.set_child(Some(&dock_content));

        debug!(
            "Window created: position={:?}, size={}x{}, layer_shell={}",
            settings.position, width, height, use_layer_shell
        );

        // Create process tracker and register apps
        let process_tracker = ProcessTracker::new();
        for app in &settings.pinned_apps {
            process_tracker.register_app(&app.command);
        }
        process_tracker.start();

        // Store dock items for later updates
        let dock_items = Rc::new(RefCell::new(Vec::new()));
        
        // Store dock items for later updates
        let dock_items_stored = Rc::clone(&dock_items);
        let magnification_stored = Rc::clone(&magnification);
        
        Self {
            window,
            dock_items: dock_items_stored,
            process_tracker,
            magnification: magnification_stored,
        }
    }

    /// Update running state for all dock items
    pub fn update_running_states(&self) {
        let dock_items = self.dock_items.borrow();
        for (command, item) in dock_items.iter() {
            let is_running = self.process_tracker.is_running(command);
            let mut item = item.borrow_mut();
            let state = if is_running {
                RunningState::Running { window_count: 1 }
            } else {
                RunningState::Stopped
            };
            item.set_running_state(state);
        }
    }

    /// Setup floating window mode (non-layer-shell)
    fn setup_floating_window(window: &ApplicationWindow, settings: &Settings) {
        // Set window size
        let (width, height) = match settings.position {
            DockPosition::Left | DockPosition::Right => {
                (settings.dock_size as i32 + 20, 520)
            }
            DockPosition::Top | DockPosition::Bottom => {
                (820, settings.dock_size as i32 + 20)
            }
        };
        
        window.set_default_size(width, height);
        
        // Keep window on top
        // Note: This may not work on all Wayland compositors
        window.set_deletable(true);
        
        info!("Floating window configured: {}x{}", width, height);
    }

    /// Present the window
    pub fn present(&self) {
        self.window.present();
    }

    /// Show settings dialog
    pub fn show_settings(&self, settings: &Settings) {
        use crate::ui::SettingsDialog;
        let dialog = SettingsDialog::new(&self.window, settings.clone());
        if let Some(new_settings) = dialog.run() {
            // Save new settings
            if let Err(e) = new_settings.save() {
                log::error!("Failed to save settings: {}", e);
            } else {
                log::info!("Settings saved successfully");
                // TODO: Reload dock with new settings
            }
        }
    }

    /// Start periodic updates for running indicators
    pub fn start_running_updates(&self) {
        use gtk::glib;
        use std::rc::Rc;
        
        let dock_items = Rc::clone(&self.dock_items);
        
        // Collect commands to check
        let commands: Vec<String> = {
            let items = dock_items.borrow();
            items.iter().map(|(cmd, _)| cmd.clone()).collect()
        };
        
        // Update running states every 2 seconds
        glib::timeout_add_local(std::time::Duration::from_secs(2), move || {
            let items = dock_items.borrow();
            for (command, item) in items.iter() {
                // Simple check: use pgrep via command
                let is_running = Self::check_process_running(command);
                let mut item = item.borrow_mut();
                let state = if is_running {
                    RunningState::Running { window_count: 1 }
                } else {
                    RunningState::Stopped
                };
                item.set_running_state(state);
            }
            glib::ControlFlow::Continue
        });
    }
    
    /// Check if a process is running (helper function)
    fn check_process_running(command: &str) -> bool {
        use std::process::Command;
        let process_name = command.split_whitespace().next().unwrap_or(command);
        Command::new("pgrep")
            .arg("-x")
            .arg(process_name)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Setup Wayland Layer Shell properties
    fn setup_layer_shell(window: &ApplicationWindow, settings: &Settings) {
        // Initialize layer shell - transforms window into layer surface
        window.init_layer_shell();
        
        // Use Overlay layer - most compatible across compositors
        window.set_layer(Layer::Overlay);

        // Simple anchor configuration - just anchor to the edge
        // Don't stretch, let the window size be natural
        match settings.position {
            DockPosition::Left => {
                window.set_anchor(Edge::Left, true);
            }
            DockPosition::Right => {
                window.set_anchor(Edge::Right, true);
            }
            DockPosition::Top => {
                window.set_anchor(Edge::Top, true);
            }
            DockPosition::Bottom => {
                window.set_anchor(Edge::Bottom, true);
            }
        }

        info!(
            "Layer shell configured: position={:?}",
            settings.position
        );
    }


    /// Create the dock content container with app items
    fn create_dock_content(
        settings: &Settings,
        dock_items: &Rc<RefCell<Vec<(String, Rc<RefCell<DockItem>>)>>>,
        magnification: &Rc<RefCell<MagnificationController>>,
    ) -> Box {
        let orientation = match settings.position {
            DockPosition::Left | DockPosition::Right => Orientation::Vertical,
            DockPosition::Top | DockPosition::Bottom => Orientation::Horizontal,
        };

        // Main container that holds everything
        let main_box = Box::builder()
            .orientation(orientation)
            .spacing(0)
            .halign(gtk::Align::Fill)
            .valign(gtk::Align::Fill)
            .vexpand(true)
            .hexpand(match settings.position {
                DockPosition::Top | DockPosition::Bottom => true,
                _ => false,
            })
            .css_classes(vec!["dock-wrapper"])
            .build();

        // Inner container for dock items with styling
        let dock_box = Box::builder()
            .orientation(orientation)
            .spacing(settings.spacing as i32)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Center)
            .css_classes(vec!["dock-container"])
            .build();

        // Add dock items for each pinned app
        for app_info in &settings.pinned_apps {
            let dock_item = Rc::new(RefCell::new(DockItem::new(app_info, settings)));
            let command = app_info.command.clone();
            dock_items.borrow_mut().push((command, Rc::clone(&dock_item)));
            dock_box.append(dock_item.borrow().widget());
        }

        main_box.append(&dock_box);

        debug!(
            "Dock content created with {} items, orientation={:?}",
            settings.pinned_apps.len(),
            orientation
        );

        main_box
    }
}


