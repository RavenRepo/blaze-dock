//! Window Exposé View
//!
//! A popup that shows all windows for an application in a grid layout.
//! Clicking a window thumbnail focuses that window.

use gtk::prelude::*;
use gtk::{Box, Button, Image, Label, Orientation};
use gtk::glib;
use log::{debug, info};
use std::rc::Rc;

use crate::services::{WindowTracker, ScreencopyService, WindowInfo};

/// Exposé view showing all windows for an app
pub struct ExposeView {
    popup: gtk::Popover,
    grid: gtk::FlowBox,
    app_id: String,
    window_tracker: Rc<WindowTracker>,
    screencopy: Rc<ScreencopyService>,
}

impl ExposeView {
    /// Create a new exposé view attached to a parent widget
    pub fn new(
        parent: &impl IsA<gtk::Widget>,
        app_id: &str,
        window_tracker: Rc<WindowTracker>,
        screencopy: Rc<ScreencopyService>,
    ) -> Self {
        let grid = gtk::FlowBox::builder()
            .orientation(Orientation::Horizontal)
            .max_children_per_line(4)
            .min_children_per_line(1)
            .selection_mode(gtk::SelectionMode::None)
            .homogeneous(true)
            .row_spacing(8)
            .column_spacing(8)
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();
        
        let popup = gtk::Popover::builder()
            .child(&grid)
            .has_arrow(true)
            .css_classes(vec!["expose-popup"])
            .build();
        
        popup.set_parent(parent);
        
        Self {
            popup,
            grid,
            app_id: app_id.to_string(),
            window_tracker,
            screencopy,
        }
    }
    
    /// Show the exposé with windows for the current app
    pub fn show(&self) {
        // Clear existing children
        while let Some(child) = self.grid.first_child() {
            self.grid.remove(&child);
        }
        
        // Get windows for this app
        let windows = self.window_tracker.get_windows_for_app(&self.app_id);
        
        if windows.is_empty() {
            // Show "No windows" message
            let label = Label::new(Some("No windows open"));
            label.add_css_class("expose-empty-label");
            self.grid.insert(&label, -1);
        } else {
            // Add window cards
            for window_info in windows {
                let card = self.create_window_card(&window_info);
                self.grid.insert(&card, -1);
            }
        }
        
        self.popup.popup();
        info!("Showing exposé for '{}' with {} windows", 
            self.app_id, 
            self.grid.observe_children().n_items()
        );
    }
    
    /// Hide the exposé
    pub fn hide(&self) {
        self.popup.popdown();
    }
    
    /// Create a card widget for a window
    fn create_window_card(&self, window: &WindowInfo) -> gtk::Widget {
        let card = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(4)
            .css_classes(vec!["expose-window-card"])
            .build();
        
        // Thumbnail or icon fallback
        let thumbnail_box = Box::builder()
            .orientation(Orientation::Vertical)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Center)
            .width_request(160)
            .height_request(100)
            .css_classes(vec!["expose-thumbnail"])
            .build();
        
        // Try to get window thumbnail
        let window_id = window.id.clone();
        let _screencopy = Rc::clone(&self.screencopy);
        let app_id = self.app_id.clone();
        
        // For now, show app icon as placeholder
        // Real thumbnails would come from screencopy service
        let icon = Image::from_icon_name(&app_id);
        icon.set_pixel_size(64);
        thumbnail_box.append(&icon);
        
        // Window title (truncated)
        let title = window.title.chars().take(25).collect::<String>();
        let title_label = Label::builder()
            .label(&title)
            .ellipsize(gtk::pango::EllipsizeMode::End)
            .max_width_chars(20)
            .css_classes(vec!["expose-window-title"])
            .build();
        
        card.append(&thumbnail_box);
        card.append(&title_label);
        
        // Make it clickable
        let button = Button::builder()
            .child(&card)
            .css_classes(vec!["expose-window-button"])
            .build();
        
        // Focus window on click
        let tracker = Rc::clone(&self.window_tracker);
        let win_id = window.id.clone();
        let popup_ref = self.popup.clone();
        
        button.connect_clicked(move |_| {
            info!("Focusing window: {}", win_id);
            tracker.focus_window(&win_id);
            popup_ref.popdown();
        });
        
        button.upcast()
    }
}

/// CSS for exposé view
pub fn get_expose_css() -> &'static str {
    r#"
    .expose-popup {
        background: alpha(@window_bg_color, 0.95);
        border-radius: 12px;
        box-shadow: 0 8px 32px rgba(0,0,0,0.3);
    }
    
    .expose-window-button {
        background: transparent;
        border: none;
        padding: 8px;
        border-radius: 8px;
        transition: background 200ms;
    }
    
    .expose-window-button:hover {
        background: alpha(@accent_bg_color, 0.3);
    }
    
    .expose-window-card {
        padding: 8px;
    }
    
    .expose-thumbnail {
        background: alpha(@window_bg_color, 0.5);
        border-radius: 6px;
        border: 1px solid alpha(@borders, 0.3);
    }
    
    .expose-window-title {
        font-size: 11px;
        color: @window_fg_color;
    }
    
    .expose-empty-label {
        padding: 20px;
        color: alpha(@window_fg_color, 0.7);
    }
    "#
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_expose_css() {
        let css = get_expose_css();
        assert!(css.contains(".expose-popup"));
        assert!(css.contains(".expose-window-button"));
    }
}
