//! Global keyboard shortcuts service
//!
//! Provides Super+1-9 shortcuts for launching/focusing dock apps
//! and keyboard navigation within the dock.

use gtk::prelude::*;
use gtk::glib;
use log::{info, debug, warn};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

/// Shortcut action types
#[derive(Debug, Clone)]
pub enum ShortcutAction {
    /// Launch or focus app at position (1-9)
    ActivateApp(u8),
    /// Toggle dock visibility
    ToggleDock,
    /// Show search/filter
    ShowSearch,
    /// Navigate left
    NavigateLeft,
    /// Navigate right
    NavigateRight,
    /// Activate focused item
    ActivateFocused,
    /// Show context menu
    ShowContextMenu,
}

/// Shortcut binding
#[derive(Debug, Clone)]
pub struct ShortcutBinding {
    pub modifiers: gtk::gdk::ModifierType,
    pub key: gtk::gdk::Key,
    pub action: ShortcutAction,
}

/// Keyboard service for global shortcuts
#[derive(Clone)]
pub struct KeyboardService {
    shortcuts: Rc<RefCell<Vec<ShortcutBinding>>>,
    action_callbacks: Rc<RefCell<HashMap<String, Box<dyn Fn(ShortcutAction)>>>>,
    enabled: Rc<RefCell<bool>>,
}

impl KeyboardService {
    /// Create a new keyboard service
    pub fn new() -> Self {
        let service = Self {
            shortcuts: Rc::new(RefCell::new(Vec::new())),
            action_callbacks: Rc::new(RefCell::new(HashMap::new())),
            enabled: Rc::new(RefCell::new(true)),
        };
        
        service.register_default_shortcuts();
        service
    }

    /// Register default shortcut bindings
    fn register_default_shortcuts(&self) {
        let mut shortcuts = self.shortcuts.borrow_mut();
        
        // Super+1-9 for app activation
        for i in 1..=9u8 {
            shortcuts.push(ShortcutBinding {
                modifiers: gtk::gdk::ModifierType::SUPER_MASK,
                key: gtk::gdk::Key::from_name(&format!("{}", i)).unwrap_or(gtk::gdk::Key::_1),
                action: ShortcutAction::ActivateApp(i),
            });
        }
        
        // Super+D to toggle dock
        shortcuts.push(ShortcutBinding {
            modifiers: gtk::gdk::ModifierType::SUPER_MASK,
            key: gtk::gdk::Key::d,
            action: ShortcutAction::ToggleDock,
        });
        
        // Super+/ for search
        shortcuts.push(ShortcutBinding {
            modifiers: gtk::gdk::ModifierType::SUPER_MASK,
            key: gtk::gdk::Key::slash,
            action: ShortcutAction::ShowSearch,
        });
        
        debug!("Registered {} default shortcuts", shortcuts.len());
    }

    /// Register action callback
    pub fn on_action<F>(&self, id: &str, callback: F)
    where
        F: Fn(ShortcutAction) + 'static,
    {
        let mut callbacks = self.action_callbacks.borrow_mut();
        callbacks.insert(id.to_string(), Box::new(callback));
    }

    /// Setup keyboard controller on a widget
    pub fn setup_keyboard_controller(&self, widget: &impl IsA<gtk::Widget>) {
        let key_controller = gtk::EventControllerKey::new();
        
        let shortcuts = Rc::clone(&self.shortcuts);
        let callbacks = Rc::clone(&self.action_callbacks);
        let enabled = Rc::clone(&self.enabled);

        key_controller.connect_key_pressed(move |_, key, _keycode, state| {
            if !*enabled.borrow() {
                return glib::Propagation::Proceed;
            }

            let shortcuts_guard = shortcuts.borrow();
            
            for binding in shortcuts_guard.iter() {
                // Check if modifiers match (ignore caps lock, num lock, etc.)
                let effective_state = state & (
                    gtk::gdk::ModifierType::SHIFT_MASK |
                    gtk::gdk::ModifierType::CONTROL_MASK |
                    gtk::gdk::ModifierType::ALT_MASK |
                    gtk::gdk::ModifierType::SUPER_MASK
                );
                
                if key == binding.key && effective_state == binding.modifiers {
                    debug!("Shortcut matched: {:?}", binding.action);
                    
                    let action = binding.action.clone();
                    let callbacks_guard = callbacks.borrow();
                    
                    for callback in callbacks_guard.values() {
                        callback(action.clone());
                    }
                    
                    return glib::Propagation::Stop;
                }
            }
            
            glib::Propagation::Proceed
        });

        widget.add_controller(key_controller);
    }

    /// Setup navigation keyboard controls
    pub fn setup_navigation(&self, widget: &impl IsA<gtk::Widget>, on_navigate: impl Fn(i32) + 'static, on_activate: impl Fn() + 'static) {
        let nav_controller = gtk::EventControllerKey::new();
        
        let on_navigate = std::rc::Rc::new(on_navigate);
        let on_activate = std::rc::Rc::new(on_activate);

        let navigate_clone = std::rc::Rc::clone(&on_navigate);
        let activate_clone = std::rc::Rc::clone(&on_activate);

        nav_controller.connect_key_pressed(move |_, key, _keycode, _state| {
            match key {
                gtk::gdk::Key::Left | gtk::gdk::Key::h => {
                    navigate_clone(-1);
                    glib::Propagation::Stop
                }
                gtk::gdk::Key::Right | gtk::gdk::Key::l => {
                    navigate_clone(1);
                    glib::Propagation::Stop
                }
                gtk::gdk::Key::Up | gtk::gdk::Key::k => {
                    navigate_clone(-1);
                    glib::Propagation::Stop
                }
                gtk::gdk::Key::Down | gtk::gdk::Key::j => {
                    navigate_clone(1);
                    glib::Propagation::Stop
                }
                gtk::gdk::Key::Return | gtk::gdk::Key::space => {
                    activate_clone();
                    glib::Propagation::Stop
                }
                _ => glib::Propagation::Proceed,
            }
        });

        widget.add_controller(nav_controller);
    }

    /// Enable or disable shortcuts
    pub fn set_enabled(&self, enabled: bool) {
        *self.enabled.borrow_mut() = enabled;
        debug!("Keyboard shortcuts {}", if enabled { "enabled" } else { "disabled" });
    }

    /// Check if shortcuts are enabled
    pub fn is_enabled(&self) -> bool {
        *self.enabled.borrow()
    }

    /// Add a custom shortcut
    pub fn add_shortcut(&self, binding: ShortcutBinding) {
        let mut shortcuts = self.shortcuts.borrow_mut();
        shortcuts.push(binding);
    }

    /// Remove shortcuts by action type
    pub fn remove_shortcuts_by_action(&self, action_type: &str) {
        let mut shortcuts = self.shortcuts.borrow_mut();
        shortcuts.retain(|b| {
            match (&b.action, action_type) {
                (ShortcutAction::ActivateApp(_), "activate") => false,
                (ShortcutAction::ToggleDock, "toggle") => false,
                (ShortcutAction::ShowSearch, "search") => false,
                _ => true,
            }
        });
    }

    /// Get all registered shortcuts
    pub fn get_shortcuts(&self) -> Vec<ShortcutBinding> {
        self.shortcuts.borrow().clone()
    }
}

impl Default for KeyboardService {
    fn default() -> Self {
        Self::new()
    }
}

/// Try to register global shortcuts via D-Bus (KDE/GNOME)
pub fn register_global_shortcuts() {
    // Try KDE Global Shortcuts
    glib::spawn_future_local(async {
        if let Err(e) = register_kde_shortcuts().await {
            warn!("Failed to register KDE shortcuts: {}", e);
        }
    });
}

/// Register shortcuts via KDE's Global Shortcuts D-Bus interface
async fn register_kde_shortcuts() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let connection = zbus::Connection::session().await?;
    
    // KDE uses org.kde.kglobalaccel
    // This is a simplified version - full implementation would need proper D-Bus interface
    info!("KDE global shortcuts registration attempted");
    
    Ok(())
}

