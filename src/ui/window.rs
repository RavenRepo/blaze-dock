//! Dock window implementation
//!
//! Creates and manages the main dock window with Wayland Layer Shell integration.

use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box, Orientation, ScrolledWindow};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use log::{debug, info, warn};

use crate::config::{DockPosition, Settings};
use crate::ui::DockItem;

/// Main dock window wrapper
pub struct DockWindow {
    window: ApplicationWindow,
}

impl DockWindow {
    /// Create a new dock window
    pub fn new(app: &Application, settings: &Settings) -> Self {
        // Check if layer shell is supported
        if !gtk4_layer_shell::is_supported() {
            warn!("Layer shell is not supported on this compositor!");
            warn!("The dock will appear as a regular window.");
        } else {
            info!("Layer shell is supported");
        }

        // Create minimal window first
        let window = ApplicationWindow::builder()
            .application(app)
            .title("BlazeDock")
            .decorated(false)
            .resizable(false)
            .build();

        // CRITICAL: Initialize layer shell IMMEDIATELY after window creation
        // and BEFORE setting any other properties
        if gtk4_layer_shell::is_supported() {
            Self::setup_layer_shell(&window, settings);
        }

        // Add CSS class for styling
        window.add_css_class("blazedock-window");

        // Now populate the dock content
        Self::populate_dock(&window, settings);

        // Set size hints after content is added
        let (width, height) = match settings.position {
            DockPosition::Left | DockPosition::Right => {
                (settings.dock_size as i32, -1)
            }
            DockPosition::Top | DockPosition::Bottom => {
                (-1, settings.dock_size as i32)
            }
        };
        
        if width > 0 {
            window.set_default_width(width);
        }
        if height > 0 {
            window.set_default_height(height);
        }

        debug!(
            "Window created with size: {}x{} for position {:?}",
            width, height, settings.position
        );
        
        Self { window }
    }

    /// Present the window
    pub fn present(&self) {
        self.window.present();
    }

    /// Setup Wayland Layer Shell properties
    /// MUST be called immediately after window construction
    fn setup_layer_shell(window: &ApplicationWindow, settings: &Settings) {
        // Initialize layer shell - this transforms the window into a layer surface
        window.init_layer_shell();
        
        // Set the layer (Top = above normal windows, below fullscreen)
        window.set_layer(Layer::Top);

        // Set namespace for the layer surface (helps compositors identify it)
        window.set_namespace("blazedock");
        
        // Configure anchors based on position
        // For a dock, we anchor to one edge and stretch along the perpendicular edges
        match settings.position {
            DockPosition::Left => {
                window.set_anchor(Edge::Left, true);
                window.set_anchor(Edge::Top, true);
                window.set_anchor(Edge::Bottom, true);
                window.set_anchor(Edge::Right, false);
                window.set_margin(Edge::Left, settings.margin as i32);
                window.set_margin(Edge::Top, settings.margin as i32);
                window.set_margin(Edge::Bottom, settings.margin as i32);
            }
            DockPosition::Right => {
                window.set_anchor(Edge::Right, true);
                window.set_anchor(Edge::Top, true);
                window.set_anchor(Edge::Bottom, true);
                window.set_anchor(Edge::Left, false);
                window.set_margin(Edge::Right, settings.margin as i32);
                window.set_margin(Edge::Top, settings.margin as i32);
                window.set_margin(Edge::Bottom, settings.margin as i32);
            }
            DockPosition::Top => {
                window.set_anchor(Edge::Top, true);
                window.set_anchor(Edge::Left, true);
                window.set_anchor(Edge::Right, true);
                window.set_anchor(Edge::Bottom, false);
                window.set_margin(Edge::Top, settings.margin as i32);
                window.set_margin(Edge::Left, settings.margin as i32);
                window.set_margin(Edge::Right, settings.margin as i32);
            }
            DockPosition::Bottom => {
                window.set_anchor(Edge::Bottom, true);
                window.set_anchor(Edge::Left, true);
                window.set_anchor(Edge::Right, true);
                window.set_anchor(Edge::Top, false);
                window.set_margin(Edge::Bottom, settings.margin as i32);
                window.set_margin(Edge::Left, settings.margin as i32);
                window.set_margin(Edge::Right, settings.margin as i32);
            }
        }

        // Enable exclusive zone if configured (reserves screen space)
        if settings.exclusive_zone {
            window.auto_exclusive_zone_enable();
            debug!("Exclusive zone enabled - windows will not overlap dock");
        }

        // Make sure keyboard interactivity is set appropriately
        // On-demand means keyboard focus only when needed (clicking buttons)
        window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::OnDemand);

        info!(
            "Layer shell configured: position={:?}, exclusive_zone={}",
            settings.position, settings.exclusive_zone
        );
    }

    /// Populate the dock with pinned applications
    fn populate_dock(window: &ApplicationWindow, settings: &Settings) {
        let orientation = match settings.position {
            DockPosition::Left | DockPosition::Right => Orientation::Vertical,
            DockPosition::Top | DockPosition::Bottom => Orientation::Horizontal,
        };

        // Create the main container for dock items
        let dock_box = Box::builder()
            .orientation(orientation)
            .spacing(settings.spacing as i32)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Center)
            .css_classes(vec!["dock-container"])
            .build();

        // Add dock items for each pinned app
        for app_info in &settings.pinned_apps {
            let dock_item = DockItem::new(app_info, settings);
            dock_box.append(dock_item.widget());
        }

        // Wrap in scrolled window for overflow handling
        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(match orientation {
                Orientation::Horizontal => gtk::PolicyType::Automatic,
                _ => gtk::PolicyType::Never,
            })
            .vscrollbar_policy(match orientation {
                Orientation::Vertical => gtk::PolicyType::Automatic,
                _ => gtk::PolicyType::Never,
            })
            .child(&dock_box)
            .build();

        window.set_child(Some(&scrolled));

        debug!(
            "Dock populated with {} pinned applications",
            settings.pinned_apps.len()
        );
    }
}

