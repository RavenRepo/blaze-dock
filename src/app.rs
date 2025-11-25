//! Application lifecycle management
//!
//! This module handles the GTK4 application lifecycle, including
//! startup, activation, and shutdown procedures.

use gtk::prelude::*;
use gtk::Application;
use log::{info, debug};
use std::cell::RefCell;
use std::rc::Rc;

use crate::config::Settings;
use crate::ui;

/// Application ID following reverse DNS convention
const APP_ID: &str = "com.blazedock.fedora";

/// Run the BlazeDock GTK4 application
///
/// # Arguments
/// * `config` - The loaded application settings
///
/// # Returns
/// Exit code (0 for success)
pub fn run_application(config: Settings) -> i32 {
    debug!("Initializing GTK4 application with ID: {}", APP_ID);

    // Create the GTK4 application instance
    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    // Wrap config in Rc<RefCell> for shared access across callbacks
    let config = Rc::new(RefCell::new(config));

    // Connect to the 'activate' signal - called when the application starts
    let config_clone = config.clone();
    app.connect_activate(move |app| {
        info!("Application activated");
        on_activate(app, config_clone.clone());
    });

    // Connect to 'startup' signal - called once before activation
    app.connect_startup(|_app| {
        info!("Application starting up");
        // Load CSS styles globally before creating windows
        ui::load_global_styles();
    });

    // Connect to 'shutdown' signal - called when the application exits
    app.connect_shutdown(|_app| {
        info!("Application shutting down");
        // Cleanup tasks can be added here
    });

    // Run the application (blocks until quit)
    app.run().into()
}

/// Handle application activation
///
/// This is called when the application is started. It creates the main
/// dock window and configures it based on user settings.
fn on_activate(app: &Application, config: Rc<RefCell<Settings>>) {
    let settings = config.borrow();
    
    // Check if a window already exists (prevents multiple windows on re-activation)
    if let Some(window) = app.active_window() {
        debug!("Window already exists, presenting it");
        window.present();
        return;
    }

    // Create the main dock window
    let window = ui::DockWindow::new(app, &settings);
    
    // Present the window
    window.present();
    
    info!("Dock window created and presented");
    
    // Start periodic updates for running indicators
    window.start_running_updates();
}

