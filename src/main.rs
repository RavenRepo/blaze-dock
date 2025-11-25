//! BlazeDock - A professional Wayland dock for Fedora 43
//!
//! This is the main entry point for the BlazeDock application.
//! It initializes logging, loads configuration, and starts the GTK4 application.

mod app;
mod config;
mod ui;
mod utils;

use anyhow::Result;
use log::{info, error};

fn main() -> Result<()> {
    // Initialize logging with environment-controlled verbosity
    // Set RUST_LOG=debug for verbose output
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    ).init();

    info!("BlazeDock v{} starting...", env!("CARGO_PKG_VERSION"));

    // Load configuration before starting the application
    let config = match config::Settings::load() {
        Ok(cfg) => {
            info!("Configuration loaded from: {:?}", config::Settings::config_path());
            cfg
        }
        Err(e) => {
            error!("Failed to load configuration: {}. Using defaults.", e);
            config::Settings::default()
        }
    };

    // Start the GTK4 application
    let exit_code = app::run_application(config);

    info!("BlazeDock exiting with code: {}", exit_code);
    
    std::process::exit(exit_code);
}

