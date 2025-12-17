//! Dock window implementation
//!
//! Creates and manages the main dock window with Wayland Layer Shell integration.

use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box, Orientation};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use log::{debug, info, warn};

use crate::config::{DockPosition, Settings};
use crate::services::{ProcessTracker, DBusService, WindowTracker, DriveMonitor, RecentFilesService};
use crate::ui::{DockItem, RunningState, MagnificationController};
use std::cell::RefCell;
use std::rc::Rc;
use tokio::sync::broadcast;

/// Main dock window wrapper
pub struct DockWindow {
    window: ApplicationWindow,
    dock_items: Rc<RefCell<Vec<(String, Rc<RefCell<DockItem>>)>>>,
    process_tracker: ProcessTracker,
    window_tracker: WindowTracker,
    drive_monitor: DriveMonitor,
    recent_files: RecentFilesService,
    magnification: Rc<RefCell<MagnificationController>>,
    dbus_service: Option<DBusService>,
    is_hidden: Rc<RefCell<bool>>,
}

impl DockWindow {
    /// Create a new dock window
    pub fn new(app: &Application, settings: &Settings) -> Self {
        let is_hidden = Rc::new(RefCell::new(false));
        
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

        // Initialize D-Bus service
        let (dbus_service, mut dbus_rx) = DBusService::new();
        dbus_service.start();

        // Create magnification controller
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

        // D-Bus event handling is currently in placeholder mode
        // TODO: Implement proper D-Bus event loop when async runtime is set up
        let _ = dbus_rx; // Acknowledge the receiver (unused for now)

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

        // Initialize window tracker
        let window_tracker = WindowTracker::new();
        window_tracker.start();

        // Initialize drive monitor
        let drive_monitor = DriveMonitor::new();
        drive_monitor.start();

        // Initialize recent files service
        let recent_files = RecentFilesService::new();
        recent_files.refresh();

        // Store dock items for later updates
        let dock_items_stored = Rc::clone(&dock_items);
        let magnification_stored = Rc::clone(&magnification);
        
        let self_instance = Self {
            window: window.clone(),
            dock_items: dock_items_stored,
            process_tracker,
            window_tracker,
            drive_monitor,
            recent_files,
            magnification: magnification_stored,
            dbus_service: Some(dbus_service),
            is_hidden: Rc::clone(&is_hidden),
        };

        // Setup auto-hide if enabled
        if settings.auto_hide {
            self_instance.setup_auto_hide(settings);
        }

        self_instance
    }

    /// Setup auto-hide functionality
    fn setup_auto_hide(&self, settings: &Settings) {
        let is_hidden_flag = Rc::clone(&self.is_hidden);
        let window = self.window.clone();
        let position = settings.position;
        
        // Initial state: visible
        window.add_css_class("dock-visible");
        
        let motion_controller = gtk::EventControllerMotion::new();
        
        let is_hidden_enter = Rc::clone(&is_hidden_flag);
        let window_enter = window.clone();
        motion_controller.connect_enter(move |_, _, _| {
            debug!("Mouse entered dock area - cancelling hide");
            *is_hidden_enter.borrow_mut() = false;
            let pos_class = format!("dock-hidden-{}", match position {
                DockPosition::Left => "left",
                DockPosition::Right => "right",
                DockPosition::Top => "top",
                DockPosition::Bottom => "bottom",
            });
            window_enter.remove_css_class(&pos_class);
            window_enter.add_css_class("dock-visible");
        });
        
        let is_hidden_leave = Rc::clone(&is_hidden_flag);
        let window_leave = window.clone();
        motion_controller.connect_leave(move |_| {
            debug!("Mouse left dock area - starting hide timer");
            *is_hidden_leave.borrow_mut() = true;
            
            let is_hidden_timer = Rc::clone(&is_hidden_leave);
            let window_timer = window_leave.clone();
            
            // Hide after 1 second of being outside
            gtk::glib::timeout_add_seconds_local(1, move || {
                // If is_hidden_timer was reset to false by enter event, don't hide
                if !*is_hidden_timer.borrow() {
                    return gtk::glib::ControlFlow::Break;
                }
                
                debug!("Auto-hiding dock");
                window_timer.remove_css_class("dock-visible");
                let pos_class = format!("dock-hidden-{}", match position {
                    DockPosition::Left => "left",
                    DockPosition::Right => "right",
                    DockPosition::Top => "top",
                    DockPosition::Bottom => "bottom",
                });
                window_timer.add_css_class(&pos_class);
                
                gtk::glib::ControlFlow::Break
            });
        });
        
        window.add_controller(motion_controller);
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

    /// Reload the dock with new settings
    pub fn reload(&self, settings: &Settings) {
        debug!("Reloading dock with new settings");
        
        // Remove old content
        self.window.set_child(None::<&gtk::Widget>);
        
        // Clear dock items
        self.dock_items.borrow_mut().clear();
        
        // Re-create content
        let dock_content = Self::create_dock_content(settings, &self.dock_items, &self.magnification);
        self.window.set_child(Some(&dock_content));
        
        // Re-setup layer shell if needed
        if gtk4_layer_shell::is_supported() && std::env::var("BLAZEDOCK_LAYER_SHELL").is_ok() {
            Self::setup_layer_shell(&self.window, settings);
        }
        
        info!("Dock reloaded successfully");
    }

    /// Show settings dialog
    pub fn show_settings(&self, settings: &Settings) {
        use crate::ui::SettingsDialog;
        let settings_clone = settings.clone();
        let dialog = SettingsDialog::new(&self.window, settings_clone);
        if let Some(new_settings) = dialog.run() {
            // Save new settings
            if let Err(e) = new_settings.save() {
                log::error!("Failed to save settings: {}", e);
            } else {
                log::info!("Settings saved successfully");
                self.reload(&new_settings);
            }
        }
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


    /// Update magnification for all dock items
    fn update_magnification_for_all(
        dock_items: &Rc<RefCell<Vec<(String, Rc<RefCell<DockItem>>)>>>,
        magnification: &Rc<RefCell<MagnificationController>>,
    ) {
        let mag = magnification.borrow();
        let hover_index = mag.hover_index();
        let items = dock_items.borrow();
        
        for (index, (_, item)) in items.iter().enumerate() {
            let scale = mag.calculate_scale(index, hover_index);
            item.borrow().set_scale(scale);
        }
    }

    /// Start periodic updates for running indicators
    pub fn start_running_updates(&self) {
        let dock_items = Rc::clone(&self.dock_items);
        let process_tracker = self.process_tracker.clone();
        let window_tracker = self.window_tracker.clone();
        
        gtk::glib::timeout_add_seconds_local(2, move || {
            let dock_items_guard = dock_items.borrow();
            for (command, item) in dock_items_guard.iter() {
                let is_running = process_tracker.is_running(command);
                
                // Get actual window count if possible
                // We extract app_id from command (simplified)
                let app_id = command.split_whitespace().next().unwrap_or(command);
                let window_count = window_tracker.get_window_count(app_id);
                
                let state = if is_running {
                    RunningState::Running { 
                        window_count: if window_count > 0 { window_count.min(255) as u8 } else { 1 } 
                    }
                } else {
                    RunningState::Stopped
                };
                item.borrow_mut().set_running_state(state);
            }
            gtk::glib::ControlFlow::Continue
        });
        
        info!("Running updates started");
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

        // Main container - CENTER aligned for proper dock positioning
        let main_box = Box::builder()
            .orientation(orientation)
            .spacing(0)
            .halign(gtk::Align::Center)  // Center horizontally
            .valign(gtk::Align::Center)  // Center vertically
            .vexpand(true)
            .hexpand(true)
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

        // Add dock items for each pinned app with magnification support
        let magnification_ref = Rc::clone(&magnification);
        let dock_items_ref = Rc::clone(&dock_items);
        
        for (index, app_info) in settings.pinned_apps.iter().enumerate() {
            let dock_item = Rc::new(RefCell::new(DockItem::new(app_info, settings)));
            let command = app_info.command.clone();
            let item_index = index;
            
            let mag_enter = Rc::clone(&magnification_ref);
            let items_enter = Rc::clone(&dock_items_ref);
            let mag_leave = Rc::clone(&magnification_ref);
            let items_leave = Rc::clone(&dock_items_ref);
            
            // Setup hover for magnification
            let item_widget = dock_item.borrow().widget().clone();
            let motion_controller = gtk::EventControllerMotion::new();
            
            motion_controller.connect_enter(move |_, _, _| {
                mag_enter.borrow_mut().set_hover(Some(item_index));
                Self::update_magnification_for_all(&items_enter, &mag_enter);
            });
            
            motion_controller.connect_leave(move |_| {
                mag_leave.borrow_mut().set_hover(None);
                Self::update_magnification_for_all(&items_leave, &mag_leave);
            });
            
            item_widget.add_controller(motion_controller);
            
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


