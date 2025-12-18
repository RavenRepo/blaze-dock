//! Trash dock item
//!
//! A special dock item that shows the system trash with empty/full state.
//! Supports drag-to-trash functionality and opens the trash folder on click.

use gtk::prelude::*;
use gtk::{Button, Image};
use gtk::gio;
use gtk::glib;
use log::{debug, info, warn};
use std::cell::RefCell;
use std::rc::Rc;

/// Trash state
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrashState {
    Empty,
    Full,
}

/// Trash dock item
pub struct TrashItem {
    button: Button,
    image: Image,
    state: Rc<RefCell<TrashState>>,
    monitor: Option<gio::FileMonitor>,
}

impl TrashItem {
    /// Create a new trash dock item
    pub fn new(icon_size: u32) -> Self {
        let state = Rc::new(RefCell::new(TrashState::Empty));
        
        // Create the image with initial empty state
        let image = Image::from_icon_name("user-trash");
        image.set_pixel_size(icon_size as i32);
        image.add_css_class("dock-item-icon");
        
        // Create button
        let button = Button::builder()
            .css_classes(vec!["dock-item", "dock-item-trash"])
            .tooltip_text("Trash")
            .child(&image)
            .build();
        
        // Setup click handler to open trash
        button.connect_clicked(|_| {
            info!("Opening trash folder");
            if let Err(e) = Self::open_trash() {
                warn!("Failed to open trash: {}", e);
            }
        });
        
        let mut trash_item = Self {
            button,
            image,
            state,
            monitor: None,
        };
        
        // Check initial state and start monitoring
        trash_item.refresh_state();
        trash_item.start_monitoring();
        
        trash_item
    }
    
    /// Get the widget
    pub fn widget(&self) -> &Button {
        &self.button
    }
    
    /// Get current state
    pub fn state(&self) -> TrashState {
        *self.state.borrow()
    }
    
    /// Refresh the trash state by counting items
    pub fn refresh_state(&self) {
        let new_state = Self::check_trash_state();
        let old_state = *self.state.borrow();
        
        if new_state != old_state {
            *self.state.borrow_mut() = new_state;
            self.update_icon();
            debug!("Trash state changed: {:?} -> {:?}", old_state, new_state);
        }
    }
    
    /// Start monitoring trash for changes
    fn start_monitoring(&mut self) {
        let trash_file = gio::File::for_uri("trash:///");
        
        match trash_file.monitor_directory(gio::FileMonitorFlags::NONE, gio::Cancellable::NONE) {
            Ok(monitor) => {
                let state = Rc::clone(&self.state);
                let image = self.image.clone();
                
                monitor.connect_changed(move |_monitor, _file, _other, event| {
                    match event {
                        gio::FileMonitorEvent::Created |
                        gio::FileMonitorEvent::Deleted |
                        gio::FileMonitorEvent::MovedIn |
                        gio::FileMonitorEvent::MovedOut => {
                            debug!("Trash changed: {:?}", event);
                            let new_state = Self::check_trash_state();
                            let old_state = *state.borrow();
                            
                            if new_state != old_state {
                                *state.borrow_mut() = new_state;
                                Self::update_icon_static(&image, new_state);
                            }
                        }
                        _ => {}
                    }
                });
                
                self.monitor = Some(monitor);
                info!("Trash monitoring started");
            }
            Err(e) => {
                warn!("Failed to monitor trash: {}", e);
            }
        }
    }
    
    /// Update the icon based on current state
    fn update_icon(&self) {
        Self::update_icon_static(&self.image, *self.state.borrow());
    }
    
    /// Static icon update (for use in closures)
    fn update_icon_static(image: &Image, state: TrashState) {
        let icon_name = match state {
            TrashState::Empty => "user-trash",
            TrashState::Full => "user-trash-full",
        };
        image.set_icon_name(Some(icon_name));
    }
    
    /// Check if trash has items
    fn check_trash_state() -> TrashState {
        let trash_file = gio::File::for_uri("trash:///");
        
        match trash_file.enumerate_children(
            "standard::name",
            gio::FileQueryInfoFlags::NONE,
            gio::Cancellable::NONE,
        ) {
            Ok(enumerator) => {
                // Check if there's at least one item
                if enumerator.next_file(gio::Cancellable::NONE).ok().flatten().is_some() {
                    TrashState::Full
                } else {
                    TrashState::Empty
                }
            }
            Err(e) => {
                debug!("Could not enumerate trash: {}", e);
                TrashState::Empty
            }
        }
    }
    
    /// Open the trash folder in file manager
    fn open_trash() -> Result<(), Box<dyn std::error::Error>> {
        std::process::Command::new("xdg-open")
            .arg("trash:///")
            .spawn()?;
        Ok(())
    }
    
    /// Empty the trash
    pub fn empty_trash(&self) {
        info!("Emptying trash...");
        
        // Use gio trash:/// to delete all items
        glib::spawn_future_local(async move {
            let trash_file = gio::File::for_uri("trash:///");
            
            match trash_file.enumerate_children(
                "standard::name",
                gio::FileQueryInfoFlags::NONE,
                gio::Cancellable::NONE,
            ) {
                Ok(enumerator) => {
                    let mut count = 0;
                    while let Ok(Some(info)) = enumerator.next_file(gio::Cancellable::NONE) {
                        let name = info.name();
                        let child = trash_file.child(&name);
                        if let Err(e) = child.delete(gio::Cancellable::NONE) {
                            warn!("Failed to delete trash item {:?}: {}", name, e);
                        } else {
                            count += 1;
                        }
                    }
                    info!("Emptied {} items from trash", count);
                }
                Err(e) => {
                    warn!("Failed to enumerate trash for emptying: {}", e);
                }
            }
        });
    }
    
    /// Setup drag-to-trash (files dropped on trash are deleted)
    pub fn setup_drop_to_delete(&self) {
        use gtk::gdk;
        
        let drop_target = gtk::DropTarget::new(glib::Type::STRING, gdk::DragAction::MOVE);
        
        drop_target.connect_drop(move |_target, value, _x, _y| {
            if let Ok(uri_str) = value.get::<String>() {
                for line in uri_str.lines() {
                    let uri = line.trim();
                    if uri.is_empty() || uri.starts_with('#') {
                        continue;
                    }
                    
                    info!("Moving to trash: {}", uri);
                    let file = gio::File::for_uri(uri);
                    
                    if let Err(e) = file.trash(gio::Cancellable::NONE) {
                        warn!("Failed to trash {}: {}", uri, e);
                    }
                }
                return true;
            }
            false
        });
        
        self.button.add_controller(drop_target);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_trash_state_default() {
        // Just verify enum works
        let state = TrashState::Empty;
        assert_eq!(state, TrashState::Empty);
        
        let full = TrashState::Full;
        assert_eq!(full, TrashState::Full);
    }
}
