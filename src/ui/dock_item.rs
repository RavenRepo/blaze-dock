//! Dock item widget
//!
//! Individual dock item representing a pinned or running application.

use gtk::prelude::*;
use gtk::{Button, Image, GestureClick};
use gtk::gdk::Rectangle;
use log::{debug, error, info};

use crate::config::{PinnedApp, Settings};
use crate::utils::launcher;
use crate::ui::{RunningIndicator, RunningState, Badge, BadgeType, BadgePosition, WindowPreview};
use std::rc::Rc;
use std::cell::RefCell;

/// A single dock item (application launcher)
pub struct DockItem {
    button: Button,
    indicator: Rc<RefCell<RunningIndicator>>,
    badge: Badge,
    preview: Rc<RefCell<WindowPreview>>,
    css_provider: gtk::CssProvider,
    app_name: String,
    app_command: String,
    app_icon: String,
    desktop_file: Option<String>,
    is_pinned: bool,
}

impl DockItem {
    /// Create a new dock item for a pinned application
    pub fn new(app: &PinnedApp, settings: &Settings) -> Self {
        let indicator = Rc::new(RefCell::new(RunningIndicator::new()));
        let badge = Badge::new(BadgeType::Count(0), BadgePosition::TopRight);
        let button = Self::create_button(app, settings, &indicator.borrow(), &badge);
        let css_provider = gtk::CssProvider::new();
        button.style_context().add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
        
        let preview = Rc::new(RefCell::new(WindowPreview::new(&button)));
        let app_name = app.name.clone();
        let app_command = app.command.clone();
        let app_icon = app.icon.clone();
        let desktop_file = app.desktop_file.clone();
        
        Self::setup_click_handler(&button, app);
        Self::setup_hover_effects(&button, settings, Rc::clone(&preview), &app_name, Rc::clone(&indicator));
        Self::setup_context_menu(&button, app, true);
        
        Self { 
            button, 
            indicator, 
            badge, 
            preview, 
            css_provider,
            app_name,
            app_command,
            app_icon,
            desktop_file,
            is_pinned: true,
        }
    }

    /// Create a new dock item for a running (non-pinned) application
    pub fn new_running(name: &str, icon: &str, command: &str, desktop_file: Option<&str>, settings: &Settings) -> Self {
        let app = PinnedApp {
            name: name.to_string(),
            icon: icon.to_string(),
            command: command.to_string(),
            desktop_file: desktop_file.map(|s| s.to_string()),
        };
        
        let indicator = Rc::new(RefCell::new(RunningIndicator::new()));
        // Set initial running state
        indicator.borrow_mut().set_state(RunningState::Running { window_count: 1 });
        
        let badge = Badge::new(BadgeType::Count(0), BadgePosition::TopRight);
        let button = Self::create_button(&app, settings, &indicator.borrow(), &badge);
        let css_provider = gtk::CssProvider::new();
        button.style_context().add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
        
        let preview = Rc::new(RefCell::new(WindowPreview::new(&button)));
        
        Self::setup_click_handler(&button, &app);
        Self::setup_hover_effects(&button, settings, Rc::clone(&preview), name, Rc::clone(&indicator));
        Self::setup_context_menu(&button, &app, false); // Not pinned
        
        Self { 
            button, 
            indicator, 
            badge, 
            preview, 
            css_provider,
            app_name: name.to_string(),
            app_command: command.to_string(),
            app_icon: icon.to_string(),
            desktop_file: desktop_file.map(|s| s.to_string()),
            is_pinned: false,
        }
    }

    /// Check if this item is pinned
    pub fn is_pinned(&self) -> bool {
        self.is_pinned
    }

    /// Get app info for pinning
    pub fn to_pinned_app(&self) -> PinnedApp {
        PinnedApp {
            name: self.app_name.clone(),
            icon: self.app_icon.clone(),
            command: self.app_command.clone(),
            desktop_file: self.desktop_file.clone(),
        }
    }

    /// Get the underlying widget
    pub fn widget(&self) -> &Button {
        &self.button
    }

    /// Update running state
    pub fn set_running_state(&mut self, state: RunningState) {
        self.indicator.borrow_mut().set_state(state);
    }

    /// Update badge
    pub fn set_badge(&mut self, badge_type: BadgeType) {
        self.badge.set_type(badge_type);
    }

    /// Set magnification scale
    pub fn set_scale(&self, scale: f64) {
        let scale_css = format!(
            ".dock-item {{ transform: scale({:.3}); }}",
            scale
        );
        self.css_provider.load_from_data(&scale_css);
    }

    /// Create the button widget with icon, indicator and badge
    fn create_button(app: &PinnedApp, settings: &Settings, indicator: &RunningIndicator, badge: &Badge) -> Button {
        let overlay = gtk::Overlay::builder().build();

        let item_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(4)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Center)
            .css_classes(vec!["dock-item-content"])
            .build();

        let image = Image::from_icon_name(&app.icon);
        image.set_pixel_size(settings.icon_size as i32);
        image.add_css_class("dock-item-icon");
        
        item_box.append(&image);
        item_box.append(indicator.widget());

        overlay.set_child(Some(&item_box));
        overlay.add_overlay(badge.widget());

        Button::builder()
            .css_classes(vec!["dock-item"])
            .tooltip_text(&app.name)
            .child(&overlay)
            .build()
    }

    /// Setup click handler to launch application
    fn setup_click_handler(button: &Button, app: &PinnedApp) {
        let command = app.command.clone();
        let name = app.name.clone();
        
        button.connect_clicked(move |_| {
            info!("Launching application: {}", name);
            
            if let Err(e) = launcher::launch_command(&command) {
                error!("Failed to launch '{}': {}", command, e);
            }
        });
    }

    /// Setup hover effects (magnification and window previews)
    fn setup_hover_effects(button: &Button, settings: &Settings, preview: Rc<RefCell<WindowPreview>>, app_name: &str, indicator: Rc<RefCell<RunningIndicator>>) {
        let motion_controller = gtk::EventControllerMotion::new();
        
        let app_name_clone = app_name.to_string();
        let preview_clone = Rc::clone(&preview);
        let indicator_clone = Rc::clone(&indicator);
        
        motion_controller.connect_enter(move |_, _, _| {
            // Show preview if app is running
            let state = indicator_clone.borrow().state();
            match state {
                RunningState::Running { window_count } | RunningState::Focused { window_count } => {
                    preview_clone.borrow().show_previews(&app_name_clone, window_count);
                }
                _ => {}
            }
        });

        let preview_leave = Rc::clone(&preview);
        motion_controller.connect_leave(move |_| {
            preview_leave.borrow().hide();
        });

        button.add_controller(motion_controller);
        
        if settings.hover_zoom {
            // magnification is handled by window-level controller
        }
    }

    /// Setup right-click context menu
    fn setup_context_menu(button: &Button, app: &PinnedApp, is_pinned: bool) {
        let gesture = GestureClick::new();
        gesture.set_button(3); // Right mouse button
        
        let app_name = app.name.clone();
        let app_icon = app.icon.clone();
        let app_command = app.command.clone();
        let app_desktop = app.desktop_file.clone();
        
        gesture.connect_released(move |gesture, _n, x, y| {
            debug!("Context menu requested for: {}", app_name);
            
            if let Some(widget) = gesture.widget() {
                // Create popover menu
                let popover = Self::create_context_menu(
                    &widget, 
                    &app_name, 
                    &app_icon, 
                    &app_command, 
                    app_desktop.as_deref(),
                    is_pinned
                );
                
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
    fn create_context_menu(
        parent: &impl IsA<gtk::Widget>, 
        app_name: &str,
        app_icon: &str,
        app_command: &str,
        desktop_file: Option<&str>,
        is_pinned: bool,
    ) -> gtk::Popover {
        let menu_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(4)
            .margin_top(8)
            .margin_bottom(8)
            .margin_start(8)
            .margin_end(8)
            .build();

        if is_pinned {
            // Unpin button for pinned apps
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
        } else {
            // Keep in Dock button for running apps
            let keep_btn = Button::builder()
                .label("Keep in Dock")
                .css_classes(vec!["context-menu-item"])
                .build();
            
            let name = app_name.to_string();
            let icon = app_icon.to_string();
            let command = app_command.to_string();
            let desktop = desktop_file.map(|s| s.to_string());
            
            keep_btn.connect_clicked(move |btn| {
                info!("Pinning app to dock: {}", name);
                
                // Load settings, add app, save
                if let Ok(mut settings) = crate::config::Settings::load() {
                    let new_app = PinnedApp {
                        name: name.clone(),
                        icon: icon.clone(),
                        command: command.clone(),
                        desktop_file: desktop.clone(),
                    };
                    settings.add_pinned_app(new_app);
                    info!("App '{}' added to dock. Restart to see changes.", name);
                }
                
                // Close the popover
                if let Some(popover) = btn.ancestor(gtk::Popover::static_type()) {
                    if let Some(p) = popover.downcast_ref::<gtk::Popover>() {
                        p.popdown();
                    }
                }
            });
            menu_box.append(&keep_btn);
        }

        // Separator
        let separator = gtk::Separator::new(gtk::Orientation::Horizontal);
        menu_box.append(&separator);

        // Edit Config button - opens config file
        let config_btn = Button::builder()
            .label("Edit Dock Config")
            .css_classes(vec!["context-menu-item"])
            .build();
        
        config_btn.connect_clicked(move |_| {
            debug!("Opening config file");
            let home = std::env::var("HOME").unwrap_or_default();
            let config_path = format!("{}/.config/blazedock/blazedock.toml", home);
            
            // Try to open with default text editor
            if let Err(e) = std::process::Command::new("xdg-open")
                .arg(&config_path)
                .spawn()
            {
                error!("Failed to open config: {}", e);
            }
        });
        menu_box.append(&config_btn);

        // Reload button
        let reload_btn = Button::builder()
            .label("Reload Dock")
            .css_classes(vec!["context-menu-item"])
            .build();
        
        reload_btn.connect_clicked(move |_| {
            info!("Reload requested - restart BlazeDock to apply changes");
        });
        menu_box.append(&reload_btn);

        let popover = gtk::Popover::builder()
            .child(&menu_box)
            .has_arrow(true)
            .build();
        
        popover.set_parent(parent);

        popover
    }
}

