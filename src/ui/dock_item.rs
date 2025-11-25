//! Dock item widget
//!
//! Individual dock item representing a pinned application.

use gtk::prelude::*;
use gtk::{Button, Image, GestureClick};
use gtk::gdk::Rectangle;
use gtk::glib;
use log::{debug, error, info};

use crate::config::{PinnedApp, Settings};
use crate::utils::launcher;

/// A single dock item (application launcher)
pub struct DockItem {
    button: Button,
}

impl DockItem {
    /// Create a new dock item for a pinned application
    pub fn new(app: &PinnedApp, settings: &Settings) -> Self {
        let button = Self::create_button(app, settings);
        Self::setup_click_handler(&button, app);
        Self::setup_hover_effects(&button, settings);
        Self::setup_context_menu(&button, app);
        
        Self { button }
    }

    /// Get the underlying widget
    pub fn widget(&self) -> &Button {
        &self.button
    }

    /// Create the button widget with icon
    fn create_button(app: &PinnedApp, settings: &Settings) -> Button {
        let button = Button::builder()
            .css_classes(vec!["dock-item"])
            .tooltip_text(&app.name)
            .build();

        // Create icon from theme or path
        let image = Image::from_icon_name(&app.icon);
        image.set_pixel_size(settings.icon_size as i32);
        image.add_css_class("dock-item-icon");
        
        button.set_child(Some(&image));

        button
    }

    /// Setup click handler to launch application
    fn setup_click_handler(button: &Button, app: &PinnedApp) {
        let command = app.command.clone();
        let name = app.name.clone();
        
        button.connect_clicked(move |_| {
            info!("Launching application: {}", name);
            
            // Launch asynchronously to prevent UI freeze
            let cmd = command.clone();
            glib::spawn_future_local(async move {
                if let Err(e) = launcher::launch_command(&cmd).await {
                    error!("Failed to launch '{}': {}", cmd, e);
                }
            });
        });
    }

    /// Setup hover zoom effects
    fn setup_hover_effects(button: &Button, settings: &Settings) {
        if !settings.hover_zoom {
            return;
        }

        let scale = settings.hover_zoom_scale;
        
        // Create event controllers for hover
        let motion_controller = gtk::EventControllerMotion::new();
        
        motion_controller.connect_enter(move |controller, _x, _y| {
            if let Some(widget) = controller.widget() {
                widget.add_css_class("dock-item-hover");
            }
        });

        motion_controller.connect_leave(move |controller| {
            if let Some(widget) = controller.widget() {
                widget.remove_css_class("dock-item-hover");
            }
        });

        button.add_controller(motion_controller);
        
        debug!("Hover effects configured with scale: {}", scale);
    }

    /// Setup right-click context menu
    fn setup_context_menu(button: &Button, app: &PinnedApp) {
        let gesture = GestureClick::new();
        gesture.set_button(3); // Right mouse button
        
        let app_name = app.name.clone();
        
        gesture.connect_released(move |gesture, _n, x, y| {
            debug!("Context menu requested for: {}", app_name);
            
            if let Some(widget) = gesture.widget() {
                // Create popover menu
                let popover = Self::create_context_menu(&widget, &app_name);
                
                // Position at click location
                popover.set_pointing_to(Some(&Rectangle::new(
                    x as i32, y as i32, 1, 1
                )));
                
                popover.popup();
            }
        });

        button.add_controller(gesture);
    }

    /// Create the context menu popover
    fn create_context_menu(parent: &impl IsA<gtk::Widget>, app_name: &str) -> gtk::Popover {
        let menu_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(4)
            .margin_top(8)
            .margin_bottom(8)
            .margin_start(8)
            .margin_end(8)
            .build();

        // Add menu items
        let unpin_btn = Button::builder()
            .label("Unpin from Dock")
            .css_classes(vec!["context-menu-item"])
            .build();
        
        let name_clone = app_name.to_string();
        unpin_btn.connect_clicked(move |_| {
            debug!("Unpin requested for: {}", name_clone);
            // TODO: Implement unpin functionality
        });

        menu_box.append(&unpin_btn);

        let popover = gtk::Popover::builder()
            .child(&menu_box)
            .has_arrow(true)
            .build();
        
        popover.set_parent(parent);

        popover
    }
}

