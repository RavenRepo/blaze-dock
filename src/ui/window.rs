//! Dock window implementation
//!
//! Creates and manages the main dock window with Wayland Layer Shell integration.

use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box, Orientation, Separator};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use log::{debug, info, warn};

use crate::config::{DockPosition, Settings, PinnedApp};
use crate::services::{
    ProcessTracker, DBusService, WindowTracker, DriveMonitor, RecentFilesService, 
    RunningAppsService, RunningApp, ThemeService, KeyboardService, ShortcutAction,
    MultiMonitorService, ScreencopyService,
};
use crate::ui::{DockItem, RunningState, MagnificationController, SearchOverlay, SearchResult};
use std::cell::RefCell;
use std::rc::Rc;

/// Main dock window wrapper
pub struct DockWindow {
    window: ApplicationWindow,
    dock_box: Rc<RefCell<Box>>,  // Inner dock container for dynamic updates
    dock_items: Rc<RefCell<Vec<(String, Rc<RefCell<DockItem>>, bool)>>>, // (command, item, is_pinned)
    running_items: Rc<RefCell<Vec<(String, Rc<RefCell<DockItem>>)>>>, // Running (non-pinned) apps
    process_tracker: ProcessTracker,
    window_tracker: WindowTracker,
    drive_monitor: DriveMonitor,
    recent_files: RecentFilesService,
    running_apps_service: Rc<RunningAppsService>,
    magnification: Rc<RefCell<MagnificationController>>,
    dbus_service: Option<DBusService>,
    is_hidden: Rc<RefCell<bool>>,
    settings: Rc<RefCell<Settings>>,
    separator: Rc<RefCell<Option<Separator>>>,
    // New services
    theme_service: ThemeService,
    keyboard_service: KeyboardService,
    multimonitor_service: MultiMonitorService,
    screencopy_service: ScreencopyService,
    focused_item_index: Rc<RefCell<Option<usize>>>,
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
        let running_items = Rc::new(RefCell::new(Vec::new()));
        let dock_box = Rc::new(RefCell::new(Box::new(Orientation::Horizontal, 0)));
        let separator: Rc<RefCell<Option<Separator>>> = Rc::new(RefCell::new(None));

        // Initialize D-Bus service
        let (dbus_service, dbus_rx) = DBusService::new();
        dbus_service.start();

        // Create magnification controller
        let magnification = Rc::new(RefCell::new(MagnificationController::new(
            settings.hover_zoom_scale,
            2,
        )));
        
        // Create dock content and store dock_box reference
        let (dock_content, inner_dock_box) = Self::create_dock_content(settings, &dock_items, &magnification);
        *dock_box.borrow_mut() = inner_dock_box;
        
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
        
        // Initialize running apps service
        let running_apps_service = Rc::new(RunningAppsService::new());
        
        // Store settings
        let settings_rc = Rc::new(RefCell::new(settings.clone()));

        // Initialize new services
        let theme_service = ThemeService::new();
        theme_service.start_monitoring();
        
        let keyboard_service = KeyboardService::new();
        let multimonitor_service = MultiMonitorService::new();
        multimonitor_service.start_monitoring();
        
        let screencopy_service = ScreencopyService::new();
        screencopy_service.start();
        
        let focused_item_index = Rc::new(RefCell::new(None::<usize>));
        
        let self_instance = Self {
            window: window.clone(),
            dock_box: Rc::clone(&dock_box),
            dock_items: dock_items_stored,
            running_items: Rc::clone(&running_items),
            process_tracker,
            window_tracker,
            drive_monitor,
            recent_files,
            running_apps_service: Rc::clone(&running_apps_service),
            magnification: magnification_stored,
            dbus_service: Some(dbus_service),
            is_hidden: Rc::clone(&is_hidden),
            settings: Rc::clone(&settings_rc),
            separator: Rc::clone(&separator),
            theme_service,
            keyboard_service,
            multimonitor_service,
            screencopy_service,
            focused_item_index: Rc::clone(&focused_item_index),
        };

        // Setup keyboard shortcuts if enabled
        if settings.enable_shortcuts {
            self_instance.setup_keyboard_shortcuts();
        }

        // Setup auto-hide if enabled
        if settings.auto_hide {
            self_instance.setup_auto_hide(settings);
        }

        self_instance
    }

    /// Setup keyboard shortcuts
    fn setup_keyboard_shortcuts(&self) {
        let dock_items = Rc::clone(&self.dock_items);
        let focused_index = Rc::clone(&self.focused_item_index);
        let window = self.window.clone();
        let settings = Rc::clone(&self.settings);
        
        // Register shortcut handler
        self.keyboard_service.on_action("main", move |action| {
            match action {
                ShortcutAction::ActivateApp(num) => {
                    let items = dock_items.borrow();
                    let index = (num as usize).saturating_sub(1);
                    if let Some((command, _, _)) = items.get(index) {
                        debug!("Activating app at index {} via shortcut", index);
                        crate::utils::launcher::launch_command(command);
                    }
                }
                ShortcutAction::ToggleDock => {
                    debug!("Toggle dock via shortcut");
                    // Toggle visibility
                    if window.is_visible() {
                        window.set_visible(false);
                    } else {
                        window.set_visible(true);
                        window.present();
                    }
                }
                ShortcutAction::ShowSearch => {
                    debug!("Show search via shortcut");
                    // TODO: Integrate search overlay
                }
                ShortcutAction::NavigateLeft | ShortcutAction::NavigateRight => {
                    let items = dock_items.borrow();
                    let mut focused = focused_index.borrow_mut();
                    
                    let direction = if matches!(action, ShortcutAction::NavigateLeft) { -1i32 } else { 1 };
                    
                    let new_index = match *focused {
                        Some(idx) => {
                            let new_idx = (idx as i32 + direction).rem_euclid(items.len() as i32) as usize;
                            Some(new_idx)
                        }
                        None => {
                            if !items.is_empty() { Some(0) } else { None }
                        }
                    };
                    
                    *focused = new_index;
                    debug!("Keyboard navigation: focused index = {:?}", new_index);
                    
                    // Update visual focus
                    for (i, (_, item, _)) in items.iter().enumerate() {
                        let widget = item.borrow().widget().clone();
                        widget.remove_css_class("dock-item-focused");
                        if new_index == Some(i) {
                            widget.add_css_class("dock-item-focused");
                        }
                    }
                }
                ShortcutAction::ActivateFocused => {
                    let items = dock_items.borrow();
                    let focused = focused_index.borrow();
                    
                    if let Some(idx) = *focused {
                        if let Some((command, _, _)) = items.get(idx) {
                            debug!("Activating focused item at index {}", idx);
                            crate::utils::launcher::launch_command(command);
                        }
                    }
                }
                _ => {}
            }
        });

        // Attach keyboard controller to window
        self.keyboard_service.setup_keyboard_controller(&self.window);
        
        info!("Keyboard shortcuts enabled");
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
        for (command, item, _is_pinned) in dock_items.iter() {
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
        window.set_deletable(true);
        
        // Enable window dragging with left mouse button
        let drag_gesture = gtk::GestureDrag::new();
        drag_gesture.set_button(1); // Left mouse button
        
        let window_weak = window.downgrade();
        drag_gesture.connect_drag_begin(move |gesture, x, y| {
            if let Some(win) = window_weak.upgrade() {
                // Get the GDK surface and initiate a move
                if let Some(native) = win.native() {
                    if let Some(surface) = native.surface() {
                        // Use the toplevel's begin_move for Wayland compatibility
                        if let Some(toplevel) = surface.downcast_ref::<gtk::gdk::Toplevel>() {
                            let device = gesture.device().unwrap();
                            let timestamp = gesture.current_event_time();
                            toplevel.begin_move(&device, 1, x, y, timestamp);
                        }
                    }
                }
            }
        });
        
        window.add_controller(drag_gesture);
        
        info!("Floating window configured: {}x{} (draggable)", width, height);
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
        
        // Clear dock items and running items
        self.dock_items.borrow_mut().clear();
        self.running_items.borrow_mut().clear();
        *self.separator.borrow_mut() = None;
        
        // Re-create content
        let (dock_content, inner_dock_box) = Self::create_dock_content(settings, &self.dock_items, &self.magnification);
        *self.dock_box.borrow_mut() = inner_dock_box;
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
        dock_items: &Rc<RefCell<Vec<(String, Rc<RefCell<DockItem>>, bool)>>>,
        magnification: &Rc<RefCell<MagnificationController>>,
    ) {
        let mag = magnification.borrow();
        let hover_index = mag.hover_index();
        let items = dock_items.borrow();
        
        for (index, (_, item, _)) in items.iter().enumerate() {
            let scale = mag.calculate_scale(index, hover_index);
            item.borrow().set_scale(scale);
        }
    }

    /// Start periodic updates for running indicators
    pub fn start_running_updates(&self) {
        let dock_items = Rc::clone(&self.dock_items);
        let running_items = Rc::clone(&self.running_items);
        let process_tracker = self.process_tracker.clone();
        let window_tracker = self.window_tracker.clone();
        
        // Update running indicators for pinned apps
        gtk::glib::timeout_add_seconds_local(2, move || {
            // Update pinned apps running state
            let dock_items_guard = dock_items.borrow();
            for (command, item, _is_pinned) in dock_items_guard.iter() {
                let is_running = process_tracker.is_running(command);
                
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
            
            // Update running (non-pinned) apps - they're always running
            let running_guard = running_items.borrow();
            for (_, item) in running_guard.iter() {
                item.borrow_mut().set_running_state(RunningState::Running { window_count: 1 });
            }
            
            gtk::glib::ControlFlow::Continue
        });
        
        info!("Running updates started");
    }

    /// Start periodic refresh of running apps
    pub fn start_running_apps_refresh(&self) {
        let dock_box = Rc::clone(&self.dock_box);
        let running_items = Rc::clone(&self.running_items);
        let separator = Rc::clone(&self.separator);
        let settings = Rc::clone(&self.settings);
        let running_apps_service = Rc::clone(&self.running_apps_service);
        
        // Refresh running apps every 3 seconds
        gtk::glib::timeout_add_seconds_local(3, move || {
            let settings_guard = settings.borrow();
            let pinned_commands: Vec<String> = settings_guard.pinned_apps.iter()
                .map(|app| app.command.clone())
                .collect();
            
            // Get currently running apps
            let running_apps = running_apps_service.get_running_apps(&pinned_commands);
            
            let dock_box_ref = dock_box.borrow();
            let mut running_items_mut = running_items.borrow_mut();
            let mut separator_mut = separator.borrow_mut();
            
            // Get current running app commands
            let current_running: std::collections::HashSet<String> = running_items_mut.iter()
                .map(|(cmd, _)| cmd.clone())
                .collect();
            
            // Get new running apps
            let new_running: std::collections::HashSet<String> = running_apps.iter()
                .map(|app| app.command.clone())
                .collect();
            
            // Remove apps that are no longer running
            running_items_mut.retain(|(cmd, item)| {
                if !new_running.contains(cmd) {
                    dock_box_ref.remove(item.borrow().widget());
                    debug!("Removed running app from dock: {}", cmd);
                    false
                } else {
                    true
                }
            });
            
            // Handle separator
            let has_running = !running_apps.is_empty();
            if !has_running {
                if let Some(sep) = separator_mut.take() {
                    dock_box_ref.remove(&sep);
                }
            } else if separator_mut.is_none() {
                let orientation = match settings_guard.position {
                    DockPosition::Left | DockPosition::Right => gtk::Orientation::Horizontal,
                    DockPosition::Top | DockPosition::Bottom => gtk::Orientation::Vertical,
                };
                let sep = Separator::builder()
                    .orientation(orientation)
                    .margin_start(8)
                    .margin_end(8)
                    .css_classes(vec!["dock-separator"])
                    .build();
                dock_box_ref.append(&sep);
                *separator_mut = Some(sep);
            }
            
            // Add new running apps
            for app in running_apps {
                if !current_running.contains(&app.command) {
                    let dock_item = Rc::new(RefCell::new(DockItem::new_running(
                        &app.name,
                        &app.icon,
                        &app.command,
                        app.desktop_file.as_deref(),
                        &settings_guard,
                    )));
                    
                    dock_box_ref.append(dock_item.borrow().widget());
                    running_items_mut.push((app.command.clone(), Rc::clone(&dock_item)));
                    
                    info!("Added running app to dock: {} ({})", app.name, app.command);
                }
            }
            
            gtk::glib::ControlFlow::Continue
        });
        
        info!("Running apps refresh started");
    }

    /// Create the dock content container with app items
    /// Returns (main_box, dock_box) so we can store dock_box for dynamic updates
    fn create_dock_content(
        settings: &Settings,
        dock_items: &Rc<RefCell<Vec<(String, Rc<RefCell<DockItem>>, bool)>>>,
        magnification: &Rc<RefCell<MagnificationController>>,
    ) -> (Box, Box) {
        let orientation = match settings.position {
            DockPosition::Left | DockPosition::Right => Orientation::Vertical,
            DockPosition::Top | DockPosition::Bottom => Orientation::Horizontal,
        };

        // Main container - CENTER aligned for proper dock positioning
        let main_box = Box::builder()
            .orientation(orientation)
            .spacing(0)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Center)
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

        // Add pinned apps
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
            
            // (command, item, is_pinned=true)
            dock_items.borrow_mut().push((command, Rc::clone(&dock_item), true));
            dock_box.append(dock_item.borrow().widget());
        }

        main_box.append(&dock_box);

        debug!(
            "Dock content created with {} pinned items, orientation={:?}",
            settings.pinned_apps.len(),
            orientation
        );

        (main_box, dock_box)
    }

    /// Refresh running apps in the dock
    pub fn refresh_running_apps(&self) {
        let settings = self.settings.borrow();
        let pinned_commands: Vec<String> = settings.pinned_apps.iter()
            .map(|app| app.command.clone())
            .collect();
        
        // Get currently running apps
        let running_apps = self.running_apps_service.get_running_apps(&pinned_commands);
        
        let dock_box = self.dock_box.borrow();
        let mut running_items = self.running_items.borrow_mut();
        let mut separator = self.separator.borrow_mut();
        
        // Get list of currently displayed running app commands
        let current_running: std::collections::HashSet<String> = running_items.iter()
            .map(|(cmd, _)| cmd.clone())
            .collect();
        
        // Get list of new running apps
        let new_running: std::collections::HashSet<String> = running_apps.iter()
            .map(|app| app.command.clone())
            .collect();
        
        // Remove apps that are no longer running
        running_items.retain(|(cmd, item)| {
            if !new_running.contains(cmd) {
                dock_box.remove(item.borrow().widget());
                debug!("Removed running app from dock: {}", cmd);
                false
            } else {
                true
            }
        });
        
        // Check if we need a separator
        let has_running = !running_apps.is_empty();
        let needs_separator = has_running && separator.is_none();
        let remove_separator = !has_running && separator.is_some();
        
        if remove_separator {
            if let Some(sep) = separator.take() {
                dock_box.remove(&sep);
            }
        }
        
        if needs_separator {
            let orientation = match settings.position {
                DockPosition::Left | DockPosition::Right => gtk::Orientation::Horizontal,
                DockPosition::Top | DockPosition::Bottom => gtk::Orientation::Vertical,
            };
            let sep = Separator::builder()
                .orientation(orientation)
                .margin_start(8)
                .margin_end(8)
                .css_classes(vec!["dock-separator"])
                .build();
            dock_box.append(&sep);
            *separator = Some(sep);
        }
        
        // Add new running apps
        for app in running_apps {
            if !current_running.contains(&app.command) {
                let dock_item = Rc::new(RefCell::new(DockItem::new_running(
                    &app.name,
                    &app.icon,
                    &app.command,
                    app.desktop_file.as_deref(),
                    &settings,
                )));
                
                dock_box.append(dock_item.borrow().widget());
                running_items.push((app.command.clone(), Rc::clone(&dock_item)));
                
                info!("Added running app to dock: {} ({})", app.name, app.command);
            }
        }
    }
}


