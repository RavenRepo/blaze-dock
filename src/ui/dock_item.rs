//! Dock item widget
//!
//! Individual dock item representing a pinned application.

use gtk::prelude::*;
use gtk::{Button, Image, GestureClick};
use gtk::gdk::Rectangle;
use log::{debug, error, info};

use crate::config::{PinnedApp, Settings};
use crate::utils::launcher;
use crate::ui::{RunningIndicator, RunningState, Badge, BadgeType, BadgePosition};

/// A single dock item (application launcher)
pub struct DockItem {
    button: Button,
    indicator: RunningIndicator,
    badge: Badge,
    css_provider: gtk::CssProvider,
}

impl DockItem {
    /// Create a new dock item for a pinned application
    pub fn new(app: &PinnedApp, settings: &Settings) -> Self {
        let indicator = RunningIndicator::new();
        let badge = Badge::new(BadgeType::Count(0), BadgePosition::TopRight);
        let button = Self::create_button(app, settings, &indicator, &badge);
        let css_provider = gtk::CssProvider::new();
        button.style_context().add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
        
        Self::setup_click_handler(&button, app);
        Self::setup_hover_effects(&button, settings);
        Self::setup_context_menu(&button, app);
        
        Self { button, indicator, badge, css_provider }
    }

    /// Get the underlying widget
    pub fn widget(&self) -> &Button {
        &self.button
    }

    /// Get the running indicator
    pub fn indicator(&mut self) -> &mut RunningIndicator {
        &mut self.indicator
    }

    /// Update running state
    pub fn set_running_state(&mut self, state: RunningState) {
        self.indicator.set_state(state);
    }

    /// Update badge
    pub fn set_badge(&mut self, badge_type: BadgeType) {
        self.badge.set_type(badge_type);
    }

    /// Set magnification scale
    pub fn set_scale(&self, scale: f64) {
        // Update the existing CSS provider instead of creating a new one
        let scale_css = format!(
            ".dock-item {{ transform: scale({:.3}); }}",
            scale
        );
        self.css_provider.load_from_data(&scale_css);
    }

    /// Create the button widget with icon, indicator and badge
    fn create_button(app: &PinnedApp, settings: &Settings, indicator: &RunningIndicator, badge: &Badge) -> Button {
        // Overlay to hold badge on top of icon
        let overlay = gtk::Overlay::builder().build();

        // Container for icon and indicator
        let item_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(4)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Center)
            .css_classes(vec!["dock-item-content"])
            .build();

        // Create icon from theme or path
        let image = Image::from_icon_name(&app.icon);
        image.set_pixel_size(settings.icon_size as i32);
        image.add_css_class("dock-item-icon");
        
        item_box.append(&image);
        
        // Add running indicator below icon
        item_box.append(indicator.widget());

        overlay.set_child(Some(&item_box));
        
        // Add badge as overlay
        overlay.add_overlay(badge.widget());

        let button = Button::builder()
            .css_classes(vec!["dock-item"])
            .tooltip_text(&app.name)
            .child(&overlay)
            .build();

        button
    }

    /// Setup click handler to launch application
    fn setup_click_handler(button: &Button, app: &PinnedApp) {
        let command = app.command.clone();
        let name = app.name.clone();
        
        button.connect_clicked(move |_| {
            info!("Launching application: {}", name);
            
            // Launch the application (spawns process and returns immediately)
            if let Err(e) = launcher::launch_command(&command) {
                error!("Failed to launch '{}': {}", command, e);
            }
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

    /// Setup middle-click to show settings (temporary - will be moved to window)
    pub fn setup_settings_shortcut(_button: &Button) {
        // TODO: Implement settings dialog trigger
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

