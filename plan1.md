This is a professional software engineering blueprint for "Project FedoraDock". This document transforms the previous code snippet into a full-scale production roadmap, designed to meet your requirement for a lag-free, error-proof application natively built for Fedora 43.
Project Overview
Target OS: Fedora 43 (Rawhide/Future)
Architecture: Wayland Native (No XWayland)
Language: Rust (for memory safety and zero-garbage-collection pauses)
GUI Toolkit: GTK4 + LibAdwaita (matches Fedora’s native aesthetics)
Protocol: Wayland Layer Shell (wlr-layer-shell-unstable-v1)
Design Goal: A "Plesk-style" vertical sidebar with glassmorphism, high performance, and system integration.
Technical Architecture & Stack
To ensure "no lag," we eliminate interpreted languages (Python/JS) and heavy frameworks (Electron).
Component	Technology	Reasoning
Core Logic	Rust	Prevents memory leaks; compiled binary ensures instant startup.
UI Rendering	GTK4	Hardware accelerated; native integration with Fedora’s GNOME desktop.
Windowing	gtk4-layer-shell	Required to make the window act as a "panel" rather than a normal window on Wayland.
App Discovery	gio / desktop-file-utils	To auto-discover installed apps without manual configuration.
Config	Serde + TOML	Fast, type-safe configuration loading.
Phased Timeline (8-Week Build Cycle)
This timeline assumes a solo developer working part-time or a small team.
Phase 1: The Core Foundation (Weeks 1-2)
Goal: A stable, empty bar that reserves screen space and survives a compositor restart.
Milestone 1.1: Setup Rust workspace and CI/CD pipeline (GitHub Actions) to auto-compile on Fedora Rawhide.
Milestone 1.2: Implement gtk4-layer-shell. Create a vertical window anchored to the left (Plesk style).
Milestone 1.3: Implement "Exclusive Zone" logic. When the dock opens, other windows (Firefox, Terminal) should shrink to fit next to it, not go under it.
Phase 2: Dynamic App Management (Weeks 3-4)
Goal: The dock automatically populates with installed applications.
Milestone 2.1: Implement a AppInfo parser. The dock scans /usr/share/applications/*.desktop.
Milestone 2.2: Create a "Pinned Apps" logic in a config.toml file.
Milestone 2.3: Implement Asynchronous Process Spawning.
Critical for "No Lag": Launching an app must happen on a separate thread so the UI never freezes, even for a millisecond.
Phase 3: The "Plesk" UI & Polish (Weeks 5-6)
Goal: High-end aesthetics and visual feedback.
Milestone 3.1: CSS Styling using GTK Inspector.
Requirement: Dark mode, slight blur (backdrop-filter), rounded corners on active items.
Milestone 3.2: State Indicators.
Draw a small white dot next to apps that are currently running. This requires querying the Wayland toplevel protocols.
Milestone 3.3: Animations.
Implement gtk::Revealer for smooth hover effects. The icon should scale up 10% on hover (using affine transformations, not CPU resizing).
Phase 4: Optimization & Packaging (Weeks 7-8)
Goal: Production-ready release.
Milestone 4.1: Resource Auditing.
Target: Idle CPU usage must be 0.0%. RAM usage under 40MB.
Milestone 4.2: Settings Menu.
A simple right-click context menu to change dock size or auto-hide preference.
Milestone 4.3: Packaging.
Build an .rpm specifically for Fedora 43.
Detailed Risk Assessment
Risk	Probability	Impact	Mitigation Strategy
Wayland Protocol Changes	Medium	High	Fedora 43 is bleeding edge. We must pin specific versions of gtk4-layer-shell and monitor upstream GNOME changes.
GPU Incompatibility	Low	Medium	Some NVIDIA drivers hate transparency on Wayland. We will implement a "fallback mode" that turns off blur if the GPU struggles.
Zombie Processes	Medium	Low	If the dock crashes, it might leave child processes. We will implement a Drop trait in Rust to ensure clean cleanup on exit.
Icon Theme Missing	Low	Low	If a user uses a custom icon pack that is incomplete, the dock looks broken. We will implement a fallback to a built-in generic SVG for missing icons.
Step-by-Step Implementation Guide (Phase 1 & 2)
Since you want to build this professionally, here is the upgraded, modular code structure you should start with today.
1. Directory Structure:
code
Text
fedora-dock/
├── Cargo.toml
├── src/
│   ├── main.rs          (Entry point)
│   ├── app.rs           (Application lifecycle)
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── window.rs    (Layer Shell logic)
│   │   └── style.css    (Plesk look-and-feel)
│   └── utils/
│       └── launcher.rs  (Async command spawner)
└── config/
    └── dock.toml        (User settings)
2. The Professional Cargo.toml:
Add optimization flags to ensure the binary is fast.
code
Toml
[package]
name = "fedora-dock"
version = "0.1.0"
edition = "2021"

[dependencies]
gtk = { version = "0.8", package = "gtk4" }
gtk4-layer-shell = "0.3"
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
directories = "5.0" # To find config paths correctly

[profile.release]
lto = true        # Link Time Optimization (makes it faster)
codegen-units = 1 # Better optimization, slower build time
strip = true      # Removes debug symbols to reduce size
opt-level = 3     # Maximum optimization
3. The "Plesk" Style (CSS):
Save this as src/ui/style.css. This mimics the clean, dark sidebar of the Plesk control panel.
code
CSS
window {
    background-color: transparent;
}

#dock-box {
    background-color: #1e2025; /* Plesk Dark Grey */
    border-right: 1px solid #333;
    padding-top: 20px;
    padding-bottom: 20px;
}

button {
    background: transparent;
    border: none;
    box-shadow: none;
    padding: 12px;
    margin: 4px 8px;
    border-radius: 8px;
    transition: background 0.2s;
}

button:hover {
    background-color: #3b404d; /* Lighter grey on hover */
}

button:active {
    background-color: #5271ff; /* Plesk Blue accent */
    color: white;
}
4. The Advanced Logic (src/main.rs):
This version includes error handling and proper application structure.
code
Rust
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Button, Box, Orientation, CssProvider};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::process::Command;

fn main() {
    let app = Application::builder()
        .application_id("com.professional.fedoradock")
        .build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
    // 1. Create Window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Fedora Dock")
        .default_width(80) // Standard sidebar width
        .build();

    // 2. Layer Shell Setup (The "Professional" part)
    window.init_layer_shell();
    window.set_layer(Layer::Top);
    
    // Anchor to Left (Plesk Sidebar Style)
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Bottom, true);
    
    // Exclusive Zone: Tells GNOME "Don't put windows under me"
    window.auto_exclusive_zone_enable();

    // 3. UI Container
    let container = Box::builder()
        .orientation(Orientation::Vertical) // Vertical for sidebar
        .spacing(10)
        .name("dock-box") // For CSS ID
        .build();

    // 4. Load Apps (Mocked for Phase 1, Dynamic in Phase 2)
    let apps = vec![
        ("firefox", "firefox"),
        ("utilities-terminal", "gnome-terminal"),
        ("system-file-manager", "nautilus"),
    ];

    for (icon, cmd) in apps {
        let btn = create_icon_button(icon, cmd);
        container.append(&btn);
    }

    window.set_child(Some(&container));

    // 5. Load CSS
    load_styles();

    window.present();
}

fn create_icon_button(icon_name: &str, command: &str) -> Button {
    let button = Button::builder().build();
    let image = gtk::Image::from_icon_name(icon_name);
    image.set_pixel_size(32);
    button.set_child(Some(&image));

    let cmd_string = command.to_string();
    
    // Async launch to prevent UI freeze
    button.connect_clicked(move |_| {
        let _ = Command::new(&cmd_string)
            .spawn()
            .map_err(|e| eprintln!("Failed to launch {}: {}", cmd_string, e));
    });

    button
}

fn load_styles() {
    let provider = CssProvider::new();
    // In production, embed this file into the binary using include_str!
    provider.load_from_data(include_str!("ui/style.css"));
    
    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().unwrap(),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
