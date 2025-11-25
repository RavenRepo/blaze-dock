This is a comprehensive, code-free architectural blueprint for building a high-performance, native dock for Fedora 43.
To achieve "no lag" and "no errors," we will follow an Agile methodology divided into 5 Sprints (2 weeks per sprint). This approach forces you to perfect one layer of the application before moving to the next, isolating bugs immediately.
Project Charter
Objective: Build a native Wayland sidebar/dock (Plesk-style) for Fedora 43.
Performance Budget: < 50MB RAM, 0% CPU at idle.
Core Protocol: Wayland Layer Shell (Essential for Fedora/GNOME).
Stability Strategy: Rust (Memory Safety) + GTK4 (Native Toolkit).
Sprint 1: The Foundation (Environment & Layer Shell)
Goal: Create an empty, unmovable window that acts as part of the desktop shell.
Task 1.1: Environment Setup
Install Fedora 43 (Rawhide) in a VM or on bare metal to ensure compatibility with the latest kernel/compositor.
Install the Rust toolchain and the gtk4-devel and gtk4-layer-shell-devel libraries via dnf.
Initialize a Git repository to track every change.
Task 1.2: The Wayland Surface
Initialize a standard GTK4 Application Window.
Crucial Step: Instead of showing the window immediately, attach the "Layer Shell" surface to it.
Configure the layer settings:
Layer: Set to Top (so it floats above wallpaper) or Overlay (if you want it above everything).
Anchor: Lock the window to the Left (for Plesk sidebar) or Bottom (for Mac dock).
Exclusive Zone: Enable auto-exclusive zone. This instructs the OS to "push" other windows aside so they don't cover your dock.
Task 1.3: The Layout Skeleton
Create a top-level container (Box) inside the window.
Set the orientation (Vertical for sidebar, Horizontal for dock).
Definition of Done: You compile and run. A blank bar appears on the screen. Maximizing a web browser causes the browser to stop at the edge of your bar, not go behind it.
Sprint 2: Core Logic (App Discovery & Async Launching)
Goal: Populate the dock with real data and launch apps without freezing the UI.
Task 2.1: The Desktop Entry Parser
Write a module that scans /usr/share/applications/ and ~/.local/share/applications/.
Filter for files ending in .desktop.
Extract the Name, Icon, and Exec fields from these files.
Optimization: Implement caching. Only scan these directories on startup or when a specific "refresh" signal is received.
Task 2.2: Icon Retrieval Strategy
Do not hardcode paths to PNGs. Use the GTK Icon Theme API.
Pass the Icon string from the desktop file to the API.
Implement a fallback logic: If the icon isn't found, load a generic "application-x-executable" icon to prevent errors.
Task 2.3: Asynchronous Process Spawning
Anti-Lag Requirement: When a user clicks an icon, the execution command must run on a separate thread or distinct process.
Use the GLib/GTK non-blocking spawn commands.
Detach the process immediately so the dock doesn't wait for the app to close.
Definition of Done: You have a bar with generic icons. Clicking one launches Firefox or Terminal instantly. The dock animation does not stutter when an app launches.
Sprint 3: The "Plesk" Aesthetic (UI & UX)
Goal: Implement the visual style using CSS and Glassmorphism.
Task 3.1: CSS Provider Integration
Attach a CSS provider to the main GTK display.
Set the main window background to transparent.
Style the internal container with a dark semi-transparent color (e.g., rgba(30, 30, 30, 0.85)).
Task 3.2: Interaction Design
Hover State: Write CSS to change the background color of a button when the mouse hovers over it.
Active State: Write CSS to darken the button when clicked.
Transitions: Use CSS transitions (not JavaScript/Rust loops) for hover effects. Set a 200ms ease-in-out transition on background-color and transform. This ensures the GPU handles the animation, not the CPU.
Task 3.3: Tooltips
Enable tooltips on the buttons.
Bind the "Name" from the desktop file to the tooltip text so users know what they are clicking.
Definition of Done: The dock looks professional. It has transparency, rounded corners, and smooth hover animations. It looks like a native part of Fedora.
Sprint 4: Configuration & Persistence
Goal: Allow users to pin apps and change settings without recompiling.
Task 4.1: Configuration Schema
Define a data structure (Struct) for settings: position, size, pinned_apps (list of strings).
Choose TOML as the file format (industry standard for Rust).
Task 4.2: Serialization/Deserialization
On startup, look for ~/.config/fedora-dock/config.toml.
If it doesn't exist, generate a default config file.
If it does exist, read it and populate the dock based on the list of pinned apps.
Task 4.3: Drag and Drop (Advanced)
Enable "Drop Target" on the dock window.
Allow users to drag an icon from the system app drawer and drop it onto the dock.
On drop, append the app name to the pinned_apps list in the config file and refresh the UI.
Definition of Done: You can close the dock, edit a text file to change the order of icons, restart the dock, and see the changes reflected.
Sprint 5: Optimization, Packaging & Release
Goal: Prepare the software for deployment on Fedora 43.
Task 5.1: The Release Build
Configure the compiler profile to Release.
Enable "Link Time Optimization" (LTO) to strip unused code and reduce binary size.
Strip debug symbols.
Task 5.2: System Integration
Create a systemd user service unit file (.service).
Configure it to WantedBy=graphical-session.target so it starts automatically when you log in.
Task 5.3: RPM Packaging
Create a .spec file. This is the blueprint for building an RPM package.
Define dependencies (gtk4, gtk4-layer-shell).
Run rpmbuild to generate a .rpm file.
Definition of Done: You have an installed .rpm. The dock starts automatically on reboot. It consumes negligible resources.
