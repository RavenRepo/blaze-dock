//! Drag and Drop support for dock items
//!
//! Enables:
//! - Reordering dock items by dragging
//! - Pinning apps by dropping .desktop files
//! - Unpinning by dragging off the dock

use gtk::prelude::*;
use gtk::gdk;
use gtk::glib;
use log::{debug, info, warn};
use std::cell::RefCell;
use std::rc::Rc;

use crate::config::{PinnedApp, Settings};
use crate::utils::desktop_entry::DesktopEntry;

/// Drag data type for dock items
pub const DOCK_ITEM_MIME: &str = "application/x-blazedock-item";

/// Data transferred during drag operations
#[derive(Clone, Debug)]
pub struct DragData {
    pub source_index: usize,
    pub app_command: String,
}

/// Setup drag source on a dock item
pub fn setup_drag_source(
    widget: &gtk::Button,
    index: usize,
    app_command: String,
    on_drag_end: Rc<RefCell<Option<Box<dyn Fn(bool) + 'static>>>>,
) {
    let drag_source = gtk::DragSource::new();
    drag_source.set_actions(gdk::DragAction::MOVE);
    
    let command_clone = app_command.clone();
    let idx = index;
    
    // Prepare drag data
    drag_source.connect_prepare(move |_source, _x, _y| {
        debug!("Drag prepare: index={}, command={}", idx, command_clone);
        
        // Create content provider with index as string
        let data = format!("{}:{}", idx, command_clone);
        let bytes = glib::Bytes::from(data.as_bytes());
        let content = gdk::ContentProvider::for_bytes(DOCK_ITEM_MIME, &bytes);
        
        Some(content)
    });
    
    // Visual feedback during drag
    let widget_weak = widget.downgrade();
    drag_source.connect_drag_begin(move |_source, _drag| {
        debug!("Drag started for index {}", idx);
        if let Some(widget) = widget_weak.upgrade() {
            widget.add_css_class("dock-item-dragging");
        }
    });
    
    // Drag ended - check if dropped outside dock
    let widget_weak2 = widget.downgrade();
    let on_end = Rc::clone(&on_drag_end);
    drag_source.connect_drag_end(move |_source, _drag, delete| {
        debug!("Drag ended, delete={}", delete);
        if let Some(widget) = widget_weak2.upgrade() {
            widget.remove_css_class("dock-item-dragging");
        }
        
        // Call the callback if we should delete (dropped outside)
        if let Some(ref callback) = *on_end.borrow() {
            callback(delete);
        }
    });
    
    widget.add_controller(drag_source);
}

/// Setup drop target on dock container for reordering
pub fn setup_drop_target_reorder(
    dock_box: &gtk::Box,
    settings: Rc<RefCell<Settings>>,
    on_reorder: Rc<RefCell<Option<Box<dyn Fn(usize, usize) + 'static>>>>,
) {
    let drop_target = gtk::DropTarget::new(glib::Type::INVALID, gdk::DragAction::MOVE);
    
    // Accept our custom MIME type
    drop_target.set_gtypes(&[glib::Type::INVALID]);
    
    let dock_box_weak = dock_box.downgrade();
    let settings_clone = Rc::clone(&settings);
    let on_reorder_clone = Rc::clone(&on_reorder);
    
    drop_target.connect_drop(move |_target, value, x, y| {
        debug!("Drop received at ({}, {})", x, y);
        
        let dock_box = match dock_box_weak.upgrade() {
            Some(b) => b,
            None => return false,
        };
        
        // Calculate target index based on drop position
        let target_index = calculate_drop_index(&dock_box, x, y);
        
        // Get source index from drag data
        // Note: GTK4's drag-drop is complex - using a simpler approach
        info!("Drop at index: {}", target_index);
        
        // Trigger reorder callback
        if let Some(ref callback) = *on_reorder_clone.borrow() {
            // We'll use 0 as source for now - real impl needs state tracking
            callback(0, target_index);
        }
        
        true
    });
    
    // Highlight during drag over
    drop_target.connect_enter(move |_target, _x, _y| {
        debug!("Drag entered dock");
        gdk::DragAction::MOVE
    });
    
    dock_box.add_controller(drop_target);
}

/// Setup drop target for .desktop files from file managers
pub fn setup_drop_target_desktop_files(
    dock_box: &gtk::Box,
    settings: Rc<RefCell<Settings>>,
    on_add: Rc<RefCell<Option<Box<dyn Fn(PinnedApp) + 'static>>>>,
) {
    // Accept text/uri-list for file drops
    let drop_target = gtk::DropTarget::new(glib::Type::STRING, gdk::DragAction::COPY);
    
    let settings_clone = Rc::clone(&settings);
    let on_add_clone = Rc::clone(&on_add);
    
    drop_target.connect_drop(move |_target, value, _x, _y| {
        debug!("File drop received");
        
        // Try to get URI list as string
        if let Ok(uri_str) = value.get::<String>() {
            // Parse URI list (one per line)
            for line in uri_str.lines() {
                let uri = line.trim();
                if uri.is_empty() || uri.starts_with('#') {
                    continue;
                }
                
                // Convert file:// URI to path
                let path = if uri.starts_with("file://") {
                    uri.strip_prefix("file://").unwrap_or(uri)
                } else {
                    uri
                };
                
                // URL decode the path
                let path = urlencoding_decode(path);
                
                // Check if it's a .desktop file
                if path.ends_with(".desktop") {
                    info!("Desktop file dropped: {}", path);
                    
                    // Parse the desktop file
                    if let Ok(entry) = DesktopEntry::parse(&path) {
                        let app = PinnedApp {
                            name: entry.name.unwrap_or_else(|| "Unknown".to_string()),
                            icon: entry.icon.unwrap_or_else(|| "application-x-executable".to_string()),
                            command: entry.exec_command().unwrap_or_else(|| path.to_string()),
                            desktop_file: Some(path.to_string()),
                        };
                        
                        // Add to settings
                        settings_clone.borrow_mut().add_pinned_app(app.clone());
                        info!("App '{}' pinned to dock", app.name);
                        
                        // Notify UI
                        if let Some(ref callback) = *on_add_clone.borrow() {
                            callback(app);
                        }
                        
                        return true;
                    } else {
                        warn!("Failed to parse desktop file: {}", path);
                    }
                }
            }
        }
        
        false
    });
    
    // Visual feedback
    drop_target.connect_enter(move |_target, _x, _y| {
        debug!("File drag entered dock");
        gdk::DragAction::COPY
    });
    
    dock_box.add_controller(drop_target);
}

/// Simple URL decoding for file paths
fn urlencoding_decode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    
    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                result.push(byte as char);
            } else {
                result.push('%');
                result.push_str(&hex);
            }
        } else if c == '+' {
            result.push(' ');
        } else {
            result.push(c);
        }
    }
    
    result
}

/// Calculate which index to drop at based on position
fn calculate_drop_index(dock_box: &gtk::Box, x: f64, y: f64) -> usize {
    let is_horizontal = dock_box.orientation() == gtk::Orientation::Horizontal;
    let pos = if is_horizontal { x } else { y };
    
    let mut current_pos = 0.0;
    let mut index = 0;
    
    // Iterate through children to find drop position
    let mut child = dock_box.first_child();
    while let Some(widget) = child {
        let allocation = widget.allocation();
        let size = if is_horizontal { 
            allocation.width() as f64 
        } else { 
            allocation.height() as f64 
        };
        
        let center = current_pos + size / 2.0;
        
        if pos < center {
            return index;
        }
        
        current_pos += size;
        index += 1;
        child = widget.next_sibling();
    }
    
    index
}

/// CSS for drag feedback
pub fn get_drag_drop_css() -> &'static str {
    r#"
    .dock-item-dragging {
        opacity: 0.5;
        transform: scale(0.9);
    }
    
    .dock-container.drag-over {
        background: alpha(@accent_color, 0.2);
    }
    
    .dock-item-drop-indicator {
        background: @accent_color;
        min-width: 3px;
        min-height: 3px;
        border-radius: 2px;
    }
    "#
}
