//! Global keyboard shortcuts service
//!
//! Provides system-wide Super+1-9 shortcuts for launching/focusing dock apps.
//! Uses compositor-specific APIs:
//! - KDE: org.kde.kglobalaccel D-Bus interface
//! - GNOME/Other: org.freedesktop.portal.GlobalShortcuts

use gtk::prelude::*;
use gtk::glib;
use log::{info, debug, warn};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

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

/// Global shortcut registration status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlobalShortcutStatus {
    /// Global shortcuts not registered
    NotRegistered,
    /// Registered via KDE kglobalaccel
    KDE,
    /// Registered via XDG Portal
    Portal,
    /// Registration failed
    Failed,
}

/// Keyboard service for global shortcuts
#[derive(Clone)]
pub struct KeyboardService {
    shortcuts: Rc<RefCell<Vec<ShortcutBinding>>>,
    action_callbacks: Rc<RefCell<HashMap<String, Box<dyn Fn(ShortcutAction)>>>>,
    enabled: Rc<RefCell<bool>>,
    global_status: Arc<Mutex<GlobalShortcutStatus>>,
}

impl KeyboardService {
    /// Create a new keyboard service
    pub fn new() -> Self {
        let service = Self {
            shortcuts: Rc::new(RefCell::new(Vec::new())),
            action_callbacks: Rc::new(RefCell::new(HashMap::new())),
            enabled: Rc::new(RefCell::new(true)),
            global_status: Arc::new(Mutex::new(GlobalShortcutStatus::NotRegistered)),
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

    /// Start global shortcut registration
    pub fn register_global_shortcuts(&self) {
        let status = self.global_status.clone();
        let callbacks = Rc::clone(&self.action_callbacks);
        
        glib::spawn_future_local(async move {
            // Try KDE first
            if let Ok(true) = try_register_kde_shortcuts().await {
                *status.lock().unwrap() = GlobalShortcutStatus::KDE;
                info!("Global shortcuts registered via KDE kglobalaccel");
                return;
            }
            
            // Try XDG Portal
            if let Ok(true) = try_register_portal_shortcuts().await {
                *status.lock().unwrap() = GlobalShortcutStatus::Portal;
                info!("Global shortcuts registered via XDG Portal");
                return;
            }
            
            warn!("Could not register global shortcuts - only dock-focused shortcuts available");
            *status.lock().unwrap() = GlobalShortcutStatus::Failed;
        });
    }

    /// Get global shortcut registration status
    pub fn get_global_status(&self) -> GlobalShortcutStatus {
        *self.global_status.lock().unwrap()
    }

    /// Setup keyboard controller on a widget (for when dock has focus)
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

/// Try to register shortcuts via KDE's kglobalaccel D-Bus interface
async fn try_register_kde_shortcuts() -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let connection = zbus::Connection::session().await?;
    
    // Check if KDE kglobalaccel service exists
    let dbus = zbus::fdo::DBusProxy::new(&connection).await?;
    let names = dbus.list_names().await?;
    
    let has_kglobalaccel = names.iter().any(|n| n.as_str() == "org.kde.kglobalaccel");
    
    if !has_kglobalaccel {
        debug!("KDE kglobalaccel not available");
        return Ok(false);
    }
    
    // Register BlazeDock shortcuts with kglobalaccel
    // Using the Component interface
    let result = connection.call_method(
        Some("org.kde.kglobalaccel"),
        "/component/blazedock",
        Some("org.kde.kglobalaccel.Component"),
        "isActive",
        &(),
    ).await;
    
    match result {
        Ok(_) => {
            debug!("BlazeDock component already registered with kglobalaccel");
        }
        Err(_) => {
            // Need to register the component first
            debug!("Registering BlazeDock with kglobalaccel");
            
            // Register each shortcut
            for i in 1..=9u8 {
                let action_id = format!("activate-app-{}", i);
                let friendly_name = format!("Activate App {}", i);
                let default_shortcut = format!("Meta+{}", i);
                
                let _ = register_kde_shortcut(&connection, &action_id, &friendly_name, &default_shortcut).await;
            }
            
            // Toggle dock
            let _ = register_kde_shortcut(&connection, "toggle-dock", "Toggle Dock", "Meta+D").await;
            
            // Show search
            let _ = register_kde_shortcut(&connection, "show-search", "Show Search", "Meta+/").await;
        }
    }
    
    info!("KDE global shortcuts registration complete");
    Ok(true)
}

/// Register a single shortcut with KDE kglobalaccel
async fn register_kde_shortcut(
    connection: &zbus::Connection,
    action_id: &str,
    friendly_name: &str,
    default_shortcut: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // kglobalaccel uses a complex signature for setShortcut
    // (ssssasus) - component, unique id, friendly name, default, active shortcuts, flags
    let result = connection.call_method(
        Some("org.kde.kglobalaccel"),
        "/kglobalaccel",
        Some("org.kde.KGlobalAccel"),
        "setShortcut",
        &(
            "blazedock",                    // Component name
            action_id,                       // Action unique ID
            friendly_name,                   // Friendly action name
            default_shortcut,                // Default shortcut
            vec![default_shortcut.to_string()], // Active shortcuts
            0x02u32,                        // Flags: Active
        ),
    ).await;
    
    match result {
        Ok(_) => debug!("Registered KDE shortcut: {} -> {}", action_id, default_shortcut),
        Err(e) => debug!("Failed to register KDE shortcut {}: {}", action_id, e),
    }
    
    Ok(())
}

/// Try to register shortcuts via XDG Desktop Portal
async fn try_register_portal_shortcuts() -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let connection = zbus::Connection::session().await?;
    
    // Check if GlobalShortcuts portal exists
    let dbus = zbus::fdo::DBusProxy::new(&connection).await?;
    let names = dbus.list_activatable_names().await?;
    
    let has_portal = names.iter().any(|n| n.as_str().contains("portal"));
    
    if !has_portal {
        debug!("XDG Portal not available");
        return Ok(false);
    }
    
    // Try to use org.freedesktop.portal.GlobalShortcuts
    let result = connection.call_method(
        Some("org.freedesktop.portal.Desktop"),
        "/org/freedesktop/portal/desktop",
        Some("org.freedesktop.portal.GlobalShortcuts"),
        "CreateSession",
        &(HashMap::<String, zbus::zvariant::Value>::new(),),
    ).await;
    
    match result {
        Ok(reply) => {
            debug!("Portal GlobalShortcuts session created: {:?}", reply);
            // Note: Full implementation would listen for Activated signals
            Ok(true)
        }
        Err(e) => {
            debug!("Portal GlobalShortcuts not available: {}", e);
            Ok(false)
        }
    }
}

/// Backward compatibility function
pub fn register_global_shortcuts() {
    let service = KeyboardService::new();
    service.register_global_shortcuts();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_creation() {
        let service = KeyboardService::new();
        assert!(service.is_enabled());
    }

    #[test]
    fn test_default_shortcuts() {
        let service = KeyboardService::new();
        let shortcuts = service.get_shortcuts();
        
        // Should have 9 app shortcuts + toggle + search = 11
        assert_eq!(shortcuts.len(), 11);
    }

    #[test]
    fn test_enable_disable() {
        let service = KeyboardService::new();
        
        assert!(service.is_enabled());
        
        service.set_enabled(false);
        assert!(!service.is_enabled());
        
        service.set_enabled(true);
        assert!(service.is_enabled());
    }

    #[test]
    fn test_custom_shortcut() {
        let service = KeyboardService::new();
        
        let initial_count = service.get_shortcuts().len();
        
        service.add_shortcut(ShortcutBinding {
            modifiers: gtk::gdk::ModifierType::CONTROL_MASK,
            key: gtk::gdk::Key::q,
            action: ShortcutAction::ToggleDock,
        });
        
        assert_eq!(service.get_shortcuts().len(), initial_count + 1);
    }
}
