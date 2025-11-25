# Fedora 43 Vertical Dock (Plesk-Style) — Step-by-Step Instructions  

These stepwise, non-code instructions will guide you in creating a professional, vertical dock for Fedora 43, supporting persistent (configurable) launchers.

---

## 1. **System Preparation**

- Ensure you are using Fedora 43 and that your system is up-to-date.
- Install core development tools and dependencies needed for GTK4, layer-shell, and Rust development.
- Verify that you are running a Wayland session, as layer-shell only works on Wayland-supported compositors.

## 2. **Install Required Packages**

- Use Fedora's package manager to install GCC, GTK4 development headers, gtk4-layer-shell headers, and Git.
- Install Rust using the official rustup installer and activate it for your environment.
- Confirm installation by checking that Rust (`rustc`) and Cargo (`cargo`) are available in your terminal.

## 3. **Setup Rust Project Structure**

- Create a new project folder for your dock.
- Use Cargo to initialize it as a Rust binary crate.
- Inside your project, add all required dependencies for GTK4 and gtk4-layer-shell.

## 4. **Configure the Project for Layer Shell and GTK4**

- Edit your project's manifest to include dependencies for GTK4, gtk4-layer-shell, and any other needed crates.
- Review layer-shell compatibility on your Fedora version and desktop environment.

## 5. **Design the Dock User Experience**

- Decide on the default position: the dock should be anchored vertically to the left edge of the screen.
- Plan for glassmorphism or clean, modern design with custom CSS for appearance.
- Lay out the arrangement for app launchers: use vertically stacked icons with configurable sizes and hover effects.
- Plan for spacing, border radius, and background effects for visual separation.

## 6. **Persistent Launcher Configuration**

- Decide on the storage format for the launcher configuration (common: a simple JSON, TOML, or YAML config file in your home directory or config folder).
- Structure the config file to map each launcher: friendly name, icon name or path, command to execute, and order in the dock.
- Plan for reading the config file at dock startup and using its contents to render the launchers.
- Plan for writing to the config file when the user adds/removes/reorders apps.

## 7. **Icon Handling and Theming**

- Choose whether to use system icon themes, bundled bitmaps, or SVG assets for app icons.
- Ensure your design supports high-DPI and adapts to various user themes.
- Plan for fallback icons when missing.

## 8. **Application Launch Functionality**

- Set the preferred method for launching desktop applications by their command, .desktop file, or through DBus (for session integration).
- Ensure each launcher supports launching the app with a click.
- Consider error handling for failed launches.

## 9. **UI Customization Options**

- Design mechanisms for users to add, remove, or reorder launchers, either by context menu or a configuration dialog.
- Provide for icon size adjustment, dock width, transparency, and border rounding in settings.
- Plan for future features such as drag-and-drop reordering or theme switching.

## 10. **Autostart Integration**

- Make the dock start automatically on login by installing a `.desktop` file into the user’s autostart directory.
- Set proper exec path and naming for your dock’s startup entry.

## 11. **Build and Install the Dock**

- Use Cargo to build an optimized release binary.
- Move the binary to a directory in your system's PATH (e.g., `/usr/local/bin`).
- Confirm the application launches from terminal and autostarts on login.

## 12. **Testing, Troubleshooting, and Refinement**

- Test on clean Fedora 43 install with multiple display DPI settings.
- Check that your dock remains visible and interactive even as windows open and close.
- Test icon sizing, application launching, and autostart.
- Debug any layer-shell/Wayland compatibility issues.

## 13. **Advanced: Extensibility and UX Polishing**

- Plan for animation effects on hover, launch, and window focus.
- Consider supporting per-workspace docks or integration with window indicators.
- Think about providing advanced config: hiding/showing on fullscreen, intelligent auto-hide, or workspace-specific launchers.
- Provide translation/localization support for your UI if necessary.

## 14. **Documentation and Distribution**

- Write a clear README to explain installation, configuration, and usage.
- Consider providing RPM packaging for easy Fedora installation.
- Publish your project on a platform like GitHub or GitLab for others to contribute and use.

---

**You now have a comprehensive, stepwise, code-free plan for building a professional, Plesk-style vertical dock for Fedora 43 with persistent, configurable launchers.**