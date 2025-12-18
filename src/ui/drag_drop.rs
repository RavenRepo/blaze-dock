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

/// Shared state for tracking drag operations
#[derive(Clone, Default)]
pub struct DragState {
    /// Index of currently dragged item (None if not dragging)
    pub dragging_index: Option<usize>,
    /// Whether drag has left the dock bounds (for unpin)
    pub outside_dock: bool,
}

/// Create shared drag state
pub fn create_drag_state() -> Rc<RefCell<DragState>> {
    Rc::new(RefCell::new(DragState::default()))
}

/// Setup drag source on a dock item for reordering
pub fn setup_drag_source_for_reorder(
    widget: &gtk::Button,
    index: usize,
    drag_state: Rc<RefCell<DragState>>,
    settings: Rc<RefCell<Settings>>,
) {
    let drag_source = gtk::DragSource::new();
    drag_source.set_actions(gdk::DragAction::MOVE);
    
    let state_prepare = Rc::clone(&drag_state);
    let idx = index;
    
    // Set dragging index when drag starts
    drag_source.connect_prepare(move |_source, _x, _y| {
        debug!("Drag prepare: item index={}", idx);
        state_prepare.borrow_mut().dragging_index = Some(idx);
        
        // Return string content with index
        let data = idx.to_string();
        let bytes = glib::Bytes::from(data.as_bytes());
        Some(gdk::ContentProvider::for_bytes("text/plain", &bytes))
    });
    
    // Visual feedback during drag
    let widget_weak = widget.downgrade();
    drag_source.connect_drag_begin(move |_source, _drag| {
        debug!("Drag started for index {}", idx);
        if let Some(widget) = widget_weak.upgrade() {
            widget.add_css_class("dock-item-dragging");
        }
    });
    
    // Handle drag end - check if dropped outside dock
    let widget_weak2 = widget.downgrade();
    let state_end = Rc::clone(&drag_state);
    let settings_clone = Rc::clone(&settings);
    let idx_for_unpin = index;
    
    drag_source.connect_drag_end(move |_source, _drag, delete_data| {
        debug!("Drag ended, delete_data={}", delete_data);
        
        if let Some(widget) = widget_weak2.upgrade() {
            widget.remove_css_class("dock-item-dragging");
        }
        
        let state = state_end.borrow();
        
        // If drag ended outside dock and delete_data is true, unpin
        if state.outside_dock || delete_data {
            info!("Item {} dragged off dock - unpinning", idx_for_unpin);
            let mut settings = settings_clone.borrow_mut();
            if idx_for_unpin < settings.pinned_apps.len() {
                let removed = settings.remove_pinned_app(idx_for_unpin);
                if let Some(app) = removed {
                    info!("Unpinned '{}' - reload dock to see changes", app.name);
                }
            }
        }
        
        // Clear drag state
        drop(state);
        state_end.borrow_mut().dragging_index = None;
        state_end.borrow_mut().outside_dock = false;
    });
    
    // Track when drag leaves widget bounds (for detecting drag-off-dock)
    drag_source.connect_drag_cancel(move |_source, _drag, _reason| {
        debug!("Drag cancelled");
        false
    });
    
    widget.add_controller(drag_source);
}

/// Setup drop target on dock container for reordering
pub fn setup_drop_target_for_reorder(
    dock_box: &gtk::Box,
    drag_state: Rc<RefCell<DragState>>,
    settings: Rc<RefCell<Settings>>,
) {
    let drop_target = gtk::DropTarget::new(glib::Type::STRING, gdk::DragAction::MOVE);
    
    let dock_box_weak = dock_box.downgrade();
    let state_drop = Rc::clone(&drag_state);
    let settings_drop = Rc::clone(&settings);
    
    drop_target.connect_drop(move |_target, _value, x, y| {
        let dock_box = match dock_box_weak.upgrade() {
            Some(b) => b,
            None => return false,
        };
        
        let state = state_drop.borrow();
        let source_index = match state.dragging_index {
            Some(idx) => idx,
            None => {
                debug!("Drop but no source index tracked");
                return false;
            }
        };
        drop(state);
        
        // Calculate target index
        let target_index = calculate_drop_index(&dock_box, x, y);
        
        if source_index == target_index {
            debug!("Source equals target, no reorder needed");
            return true;
        }
        
        info!("Reordering: {} -> {}", source_index, target_index);
        
        // Reorder in settings
        settings_drop.borrow_mut().reorder_pinned_app(source_index, target_index);
        info!("Reorder saved - reload dock to see changes");
        
        // Mark drop successful (not outside dock)
        state_drop.borrow_mut().outside_dock = false;
        
        true
    });
    
    // Track when drag is inside dock
    let state_enter = Rc::clone(&drag_state);
    drop_target.connect_enter(move |_target, _x, _y| {
        debug!("Drag entered dock area");
        state_enter.borrow_mut().outside_dock = false;
        gdk::DragAction::MOVE
    });
    
    // Track when drag leaves dock (for unpin)
    let state_leave = Rc::clone(&drag_state);
    drop_target.connect_leave(move |_target| {
        debug!("Drag left dock area - will unpin if dropped");
        state_leave.borrow_mut().outside_dock = true;
    });
    
    dock_box.add_controller(drop_target);
}

/// Setup drop target for .desktop files from file managers
/// Drops are automatically saved to config - caller should reload dock to see changes
pub fn setup_drop_target_desktop_files(
    dock_box: &gtk::Box,
    settings: Rc<RefCell<Settings>>,
) {
    // Accept text/uri-list for file drops
    let drop_target = gtk::DropTarget::new(glib::Type::STRING, gdk::DragAction::COPY);
    
    let settings_clone = Rc::clone(&settings);
    
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
                        let name = entry.name.clone().unwrap_or_else(|| "Unknown".to_string());
                        let icon = entry.icon.clone().unwrap_or_else(|| "application-x-executable".to_string());
                        let command = entry.exec_command().unwrap_or_else(|| path.to_string());
                        
                        let app = PinnedApp {
                            name: name.clone(),
                            icon,
                            command,
                            desktop_file: Some(path.to_string()),
                        };
                        
                        // Add to settings and save
                        settings_clone.borrow_mut().add_pinned_app(app);
                        info!("App '{}' pinned to dock - reload to see changes", name);
                        
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
    "#
}
