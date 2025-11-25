//! Settings dialog
//!
//! GUI for configuring BlazeDock settings.

use gtk::prelude::*;
use gtk::{ComboBoxText, Dialog, Scale, Switch, Window};
use log::debug;

use crate::config::{DockPosition, Settings};

/// Settings dialog window
pub struct SettingsDialog {
    dialog: Dialog,
    position_combo: ComboBoxText,
    icon_size_scale: Scale,
    dock_size_scale: Scale,
    opacity_scale: Scale,
    auto_hide_switch: Switch,
    hover_zoom_switch: Switch,
    hover_zoom_scale: Scale,
    settings: Settings,
}

impl SettingsDialog {
    /// Create a new settings dialog
    pub fn new(parent: &impl IsA<Window>, settings: Settings) -> Self {
        let dialog = Dialog::builder()
            .title("BlazeDock Settings")
            .modal(true)
            .resizable(false)
            .build();
        
        dialog.set_transient_for(Some(parent));

        // Create content area
        let content = dialog.content_area();
        content.set_spacing(12);
        content.set_margin_top(12);
        content.set_margin_bottom(12);
        content.set_margin_start(12);
        content.set_margin_end(12);
        
        // Use Box for layout instead of content_area directly
        let main_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(12)
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();

        // Position selector
        let position_label = gtk::Label::new(Some("Position:"));
        position_label.set_halign(gtk::Align::Start);
        let position_combo = ComboBoxText::new();
        position_combo.append_text("Left");
        position_combo.append_text("Right");
        position_combo.append_text("Top");
        position_combo.append_text("Bottom");
        
        match settings.position {
            DockPosition::Left => position_combo.set_active(Some(0)),
            DockPosition::Right => position_combo.set_active(Some(1)),
            DockPosition::Top => position_combo.set_active(Some(2)),
            DockPosition::Bottom => position_combo.set_active(Some(3)),
        }

        // Icon size
        let icon_size_label = gtk::Label::new(Some(&format!("Icon Size: {}px", settings.icon_size)));
        icon_size_label.set_halign(gtk::Align::Start);
        let icon_size_scale = Scale::builder()
            .orientation(gtk::Orientation::Horizontal)
            .adjustment(&gtk::Adjustment::new(
                settings.icon_size as f64,
                24.0,
                128.0,
                4.0,
                8.0,
                0.0,
            ))
            .digits(0)
            .build();
        
        let icon_size_label_clone = icon_size_label.clone();
        icon_size_scale.connect_value_changed(move |scale| {
            let value = scale.value() as u32;
            icon_size_label_clone.set_text(&format!("Icon Size: {}px", value));
        });

        // Dock size
        let dock_size_label = gtk::Label::new(Some(&format!("Dock Size: {}px", settings.dock_size)));
        dock_size_label.set_halign(gtk::Align::Start);
        let dock_size_scale = Scale::builder()
            .orientation(gtk::Orientation::Horizontal)
            .adjustment(&gtk::Adjustment::new(
                settings.dock_size as f64,
                48.0,
                200.0,
                4.0,
                8.0,
                0.0,
            ))
            .digits(0)
            .build();
        
        let dock_size_label_clone = dock_size_label.clone();
        dock_size_scale.connect_value_changed(move |scale| {
            let value = scale.value() as u32;
            dock_size_label_clone.set_text(&format!("Dock Size: {}px", value));
        });

        // Opacity
        let opacity_label = gtk::Label::new(Some(&format!("Opacity: {:.0}%", settings.opacity * 100.0)));
        opacity_label.set_halign(gtk::Align::Start);
        let opacity_scale = Scale::builder()
            .orientation(gtk::Orientation::Horizontal)
            .adjustment(&gtk::Adjustment::new(
                settings.opacity,
                0.0,
                1.0,
                0.05,
                0.1,
                0.0,
            ))
            .digits(2)
            .build();
        
        let opacity_label_clone = opacity_label.clone();
        opacity_scale.connect_value_changed(move |scale| {
            let value = scale.value();
            opacity_label_clone.set_text(&format!("Opacity: {:.0}%", value * 100.0));
        });

        // Auto-hide
        let auto_hide_switch = Switch::builder()
            .active(settings.auto_hide)
            .halign(gtk::Align::Start)
            .build();
        let auto_hide_label = gtk::Label::new(Some("Auto-hide"));
        auto_hide_label.set_halign(gtk::Align::Start);

        // Hover zoom
        let hover_zoom_switch = Switch::builder()
            .active(settings.hover_zoom)
            .halign(gtk::Align::Start)
            .build();
        let hover_zoom_label = gtk::Label::new(Some("Hover Zoom"));
        hover_zoom_label.set_halign(gtk::Align::Start);

        // Hover zoom scale
        let hover_zoom_scale_label = gtk::Label::new(Some(&format!("Zoom Scale: {:.2}x", settings.hover_zoom_scale)));
        hover_zoom_scale_label.set_halign(gtk::Align::Start);
        let hover_zoom_scale = Scale::builder()
            .orientation(gtk::Orientation::Horizontal)
            .adjustment(&gtk::Adjustment::new(
                settings.hover_zoom_scale,
                1.0,
                2.0,
                0.05,
                0.1,
                0.0,
            ))
            .digits(2)
            .build();
        
        let hover_zoom_scale_label_clone = hover_zoom_scale_label.clone();
        hover_zoom_scale.connect_value_changed(move |scale| {
            let value = scale.value();
            hover_zoom_scale_label_clone.set_text(&format!("Zoom Scale: {:.2}x", value));
        });

        // Layout controls
        let position_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(12)
            .build();
        position_box.append(&position_label);
        position_box.append(&position_combo);

        let icon_size_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(4)
            .build();
        icon_size_box.append(&icon_size_label);
        icon_size_box.append(&icon_size_scale);

        let dock_size_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(4)
            .build();
        dock_size_box.append(&dock_size_label);
        dock_size_box.append(&dock_size_scale);

        let opacity_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(4)
            .build();
        opacity_box.append(&opacity_label);
        opacity_box.append(&opacity_scale);

        let auto_hide_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(12)
            .build();
        auto_hide_box.append(&auto_hide_label);
        auto_hide_box.append(&auto_hide_switch);

        let hover_zoom_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(12)
            .build();
        hover_zoom_box.append(&hover_zoom_label);
        hover_zoom_box.append(&hover_zoom_switch);

        let hover_zoom_scale_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(4)
            .build();
        hover_zoom_scale_box.append(&hover_zoom_scale_label);
        hover_zoom_scale_box.append(&hover_zoom_scale);

        // Add all to main box
        main_box.append(&position_box);
        main_box.append(&icon_size_box);
        main_box.append(&dock_size_box);
        main_box.append(&opacity_box);
        main_box.append(&auto_hide_box);
        main_box.append(&hover_zoom_box);
        main_box.append(&hover_zoom_scale_box);
        
        // Set content
        content.append(&main_box);

        // Note: GTK4 Dialog buttons are added differently
        // For now, we'll use a simpler approach

        Self {
            dialog,
            position_combo,
            icon_size_scale,
            dock_size_scale,
            opacity_scale,
            auto_hide_switch,
            hover_zoom_switch,
            hover_zoom_scale,
            settings,
        }
    }

    /// Show the dialog and return updated settings if OK/Apply was clicked
    pub fn run(&self) -> Option<Settings> {
        self.dialog.present();
        // For now, return current settings
        // TODO: Implement proper modal dialog with response handling
        Some(self.get_settings())
    }
    
    /// Get the dialog widget
    pub fn widget(&self) -> &Dialog {
        &self.dialog
    }

    /// Get current settings from dialog
    fn get_settings(&self) -> Settings {
        let position = match self.position_combo.active() {
            Some(0) => DockPosition::Left,
            Some(1) => DockPosition::Right,
            Some(2) => DockPosition::Top,
            Some(3) => DockPosition::Bottom,
            _ => self.settings.position,
        };

        let mut new_settings = self.settings.clone();
        new_settings.position = position;
        new_settings.icon_size = self.icon_size_scale.value() as u32;
        new_settings.dock_size = self.dock_size_scale.value() as u32;
        new_settings.opacity = self.opacity_scale.value();
        new_settings.auto_hide = self.auto_hide_switch.is_active();
        new_settings.hover_zoom = self.hover_zoom_switch.is_active();
        new_settings.hover_zoom_scale = self.hover_zoom_scale.value();

        new_settings
    }
}

