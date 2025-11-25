To create a professional, lag-free dock for Fedora 43 (which is currently a future/development release, with Fedora 41 being the current stable version), you need to build a native application that supports **Wayland**. Fedora defaults to Wayland, and older X11 docks (like Plank) often glitch or do not work at all on it.

I will interpret "Plesk type dock" as a typo for **"Plank"** (the popular, simple macOS-style dock), but I will design the code so it can also function as a sidebar (which is what the actual Plesk control panel looks like) if you prefer that.

To build this professionally without errors or lag, we will use **Rust** and **GTK4** with the **Wayland Layer Shell** protocol. This combination is memory-safe, extremely fast (compiled), and native to the Fedora desktop environment.

### Prerequisites

You need to set up a development environment. Open your terminal and run the following commands to install the Rust compiler and necessary system libraries.

```bash
# 1. Update your system
sudo dnf update -y

# 2. Install build tools, GTK4, and Layer Shell development headers
sudo dnf install -y gcc gtk4-devel gtk4-layer-shell-devel curl git

# 3. Install Rust (Select option 1 'Proceed with installation' when prompted)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 4. Activate Rust in the current session
source "$HOME/.cargo/env"
```

### Step 1: Create the Project Structure

We will create a new Rust binary project.

```bash
# Create a new folder for your dock
mkdir my-fedora-dock
cd my-fedora-dock

# Initialize the Rust project
cargo init
```

### Step 2: Configure Dependencies

Open the `Cargo.toml` file in your text editor. This file tells the compiler which libraries to use. We need `gtk4` for the widgets and `gtk4-layer-shell` to make the window behave like a dock (stick to the bottom, not overlap other windows).

**File:** `Cargo.toml`
```toml
[package]
name = "my-fedora-dock"
version = "0.1.0"
edition = "2021"

[dependencies]
gtk = { version = "0.8", package = "gtk4" }
gtk4-layer-shell = "0.3"
```

### Step 3: Write the Dock Code

Now we will write the actual application logic. Open `src/main.rs` and replace its contents with the code below.

This code creates a window, tells Wayland "this is a dock" (so it reserves space on the screen), and adds a few sample app launchers.

**File:** `src/main.rs`
```rust
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Button, Box, Orientation, IconTheme, CssProvider, StyleContext};
use gtk4_layer_shell::{Edge, Layer, LayerShell};

fn main() {
    // Create the application structure
    let app = Application::builder()
        .application_id("com.example.fedora_dock")
        .build();

    // When the app starts, run the 'build_ui' function
    app.connect_activate(build_ui);

    // Run the application
    app.run();
}

fn build_ui(app: &Application) {
    // 1. Create the main window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Fedora Dock")
        .build();

    // 2. Initialize Layer Shell (Critical for Wayland Docks)
    // This tells the compositor (GNOME) that this is a panel, not a normal window.
    window.init_layer_shell();
    window.set_layer(Layer::Top); // Keep it above desktop wallpaper
    
    // Position the dock at the bottom of the screen
    window.set_anchor(Edge::Bottom, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, true);

    // Reserve space so maximized windows don't cover it (Optional: comment out to make it float over windows)
    window.auto_exclusive_zone_enable();

    // Set margins to make it look like a floating dock (Plank style)
    window.set_margin_bottom(10);
    window.set_margin_start(200); // Centerish approximation
    window.set_margin_end(200);

    // 3. Create the container for icons
    let container = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(15)
        .halign(gtk::Align::Center) // Center the icons inside the dock
        .css_classes(vec!["dock-container"])
        .build();

    // 4. Add App Launchers (Buttons with Icons)
    let apps = vec![
        ("firefox", "firefox"),      // (Icon Name, Command)
        ("org.gnome.Terminal", "gnome-terminal"),
        ("org.gnome.Nautilus", "nautilus"),
        ("system-settings", "gnome-control-center"),
    ];

    for (icon_name, command) in apps {
        let button = Button::builder()
            .css_classes(vec!["dock-icon"])
            .build();
        
        // Add icon to button
        let icon = gtk::Image::from_icon_name(icon_name);
        icon.set_pixel_size(48); // Large icons
        button.set_child(Some(&icon));

        // Add click behavior
        let cmd_string = command.to_string();
        button.connect_clicked(move |_| {
            // Spawn the command asynchronously
            let _ = std::process::Command::new(&cmd_string).spawn();
        });

        container.append(&button);
    }

    window.set_child(Some(&container));

    // 5. Apply Custom CSS (Styling)
    load_css();

    window.present();
}

fn load_css() {
    // Define the visual style (Glassmorphism / "Plesk" Clean look)
    let css_data = "
        window {
            background-color: transparent;
        }
        .dock-container {
            background-color: rgba(0, 0, 0, 0.75); /* Dark semi-transparent background */
            border-radius: 20px;
            padding: 10px 20px;
            border: 1px solid rgba(255, 255, 255, 0.1);
        }
        .dock-icon {
            background: transparent;
            border: none;
            box-shadow: none;
            padding: 5px;
            transition: all 0.2s ease-in-out;
        }
        .dock-icon:hover {
            background-color: rgba(255, 255, 255, 0.2);
            border-radius: 12px;
            transform: scale(1.1); /* Zoom effect on hover */
        }
    ";

    let provider = CssProvider::new();
    provider.load_from_data(css_data);

    // Apply CSS to the default display
    if let Some(display) = gtk::gdk::Display::default() {
        StyleContext::add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
```

### Step 4: Compile and Run

This step will compile your code into a highly optimized machine binary. The first compile will take a minute or two as it builds the GTK libraries.

```bash
cargo run --release
```

If successful, a dark, glassy dock will appear at the bottom of your screen. It will support:
1.  **Wayland Native:** It won't flicker or lag like X11 docks.
2.  **Zoom Effect:** Hover over icons to see them scale up slightly.
3.  **Launching:** Clicking icons will launch the respective apps.

### Step 5: "Professionalizing" the Dock

To make this a permanent part of your system:

1.  **Build the Release Binary:**
    ```bash
    cargo build --release
    ```
2.  **Move to System Path:**
    ```bash
    sudo cp target/release/my-fedora-dock /usr/local/bin/
    ```
3.  **Auto-start on Login:**
    Create a file at `~/.config/autostart/my-dock.desktop`:
    ```ini
    [Desktop Entry]
    Type=Application
    Name=My Fedora Dock
    Exec=/usr/local/bin/my-fedora-dock
    Hidden=false
    NoDisplay=false
    X-GNOME-Autostart-enabled=true
    ```

### Troubleshooting & Customization

*   **Plesk Sidebar Style:** If you actually wanted a vertical sidebar (like the Plesk UI), change these lines in `src/main.rs`:
    *   Change `window.set_anchor(Edge::Bottom, true);` to `window.set_anchor(Edge::Left, true);`
    *   Change `window.set_anchor(Edge::Right, true);` to `window.set_anchor(Edge::Top, true);`
    *   Change `container.orientation(Orientation::Horizontal)` to `Orientation::Vertical`.
*   **Icons Missing:** Ensure you have an icon theme installed (Fedora default is Adwaita, which is included). If icons are blank, check the "Icon Name" in the code against `/usr/share/icons/`.
