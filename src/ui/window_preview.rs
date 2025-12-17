//! Window preview popover
//!
//! Shows thumbnails of open windows when hovering over dock items.

use gtk::prelude::*;
use gtk::{Box, Label, Picture, Popover, Widget, Button};
use log::debug;

/// Window preview component
pub struct WindowPreview {
    popover: Popover,
    content: Box,
}

impl WindowPreview {
    /// Create a new window preview popover
    pub fn new(parent: &impl IsA<Widget>) -> Self {
        let content = Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(8)
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .css_classes(vec!["window-preview-content"])
            .build();

        let popover = Popover::builder()
            .child(&content)
            .has_arrow(true)
            .autohide(false) // We manage visibility manually via hover
            .css_classes(vec!["window-preview-popover"])
            .build();

        // GTK4-rs PopoverExt::set_parent takes &impl IsA<Widget> directly, NOT Option
        popover.set_parent(parent);

        Self { popover, content }
    }

    /// Show previews for an application
    pub fn show_previews(&self, app_name: &str, window_count: u8) {
        // Clear old content
        while let Some(child) = self.content.first_child() {
            self.content.remove(&child);
        }

        // Header: App Name
        let header = Label::builder()
            .label(app_name)
            .halign(gtk::Align::Start)
            .css_classes(vec!["window-preview-header"])
            .build();
        self.content.append(&header);

        // Previews container (horizontal if multiple windows)
        let previews_box = Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(12)
            .build();

        // Create mock previews for now (Sprint 5 foundation)
        // In Sprint 5.2, we will replace this with real screencopy thumbnails
        for i in 0..window_count {
            let item = self.create_preview_item(&format!("Window {}", i + 1));
            previews_box.append(&item);
        }

        self.content.append(&previews_box);
        
        debug!("Showing {} previews for {}", window_count, app_name);
        self.popover.popup();
    }

    /// Hide the preview
    pub fn hide(&self) {
        self.popover.popdown();
    }

    /// Create a single preview item
    fn create_preview_item(&self, title: &str) -> Box {
        let container = Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(4)
            .css_classes(vec!["window-preview-item"])
            .build();

        // Mock thumbnail (placeholder)
        let thumbnail = Picture::builder()
            .width_request(160)
            .height_request(100)
            .css_classes(vec!["window-preview-thumbnail"])
            .build();
        
        let label = Label::builder()
            .label(title)
            .halign(gtk::Align::Center)
            .max_width_chars(20)
            .ellipsize(gtk::pango::EllipsizeMode::End)
            .css_classes(vec!["window-preview-title"])
            .build();

        container.append(&thumbnail);
        container.append(&label);
        container
    }
}
