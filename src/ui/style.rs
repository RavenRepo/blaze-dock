//! CSS styling for BlazeDock
//!
//! Loads and applies the glassmorphism/Plesk-style appearance.

use gtk::CssProvider;
use log::{debug, warn};

/// CSS styles embedded in the binary
const STYLES: &str = include_str!("style.css");

/// Load global CSS styles for the application
pub fn load_global_styles() {
    let provider = CssProvider::new();
    provider.load_from_data(STYLES);
    
    if let Some(display) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        debug!("Global CSS styles loaded successfully");
    } else {
        warn!("Failed to get default display for CSS provider");
    }
}

