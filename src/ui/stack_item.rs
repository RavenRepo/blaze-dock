//! Downloads Stack (Folder Stack)
//!
//! A macOS-style stack dock item that shows contents of a folder
//! (typically ~/Downloads) in a fan or grid popup.

use gtk::prelude::*;
use gtk::{Button, Image, Label, Box, Orientation, ScrolledWindow};
use gtk::gio::{self, FileMonitorEvent};
use gtk::glib;
use log::{debug, info, warn};
use std::cell::RefCell;
use std::rc::Rc;
use std::path::PathBuf;

/// View mode for the stack popup
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum StackViewMode {
    #[default]
    Fan,
    Grid,
    List,
}

/// A file entry in the stack
#[derive(Clone, Debug)]
pub struct StackEntry {
    pub name: String,
    pub path: PathBuf,
    pub icon_name: String,
    pub is_directory: bool,
    pub modified: Option<glib::DateTime>,
}

/// Downloads/folder stack dock item
pub struct StackItem {
    button: Button,
    icon: Image,
    popup: gtk::Popover,
    folder_path: PathBuf,
    entries: Rc<RefCell<Vec<StackEntry>>>,
    view_mode: Rc<RefCell<StackViewMode>>,
    monitor: Option<gio::FileMonitor>,
    max_items: usize,
}

impl StackItem {
    /// Create a new stack item for the given folder
    pub fn new(folder_path: PathBuf, icon_size: u32) -> Self {
        let entries = Rc::new(RefCell::new(Vec::new()));
        let view_mode = Rc::new(RefCell::new(StackViewMode::default()));
        
        // Icon shows folder or top file
        let icon = Image::from_icon_name("folder-download");
        icon.set_pixel_size(icon_size as i32);
        icon.add_css_class("dock-item-icon");
        
        // Folder name as tooltip
        let folder_name = folder_path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Downloads".to_string());
        
        let button = Button::builder()
            .css_classes(vec!["dock-item", "dock-item-stack"])
            .tooltip_text(&folder_name)
            .child(&icon)
            .build();
        
        // Create popup for showing files
        let popup = Self::create_popup();
        popup.set_parent(&button);
        
        let mut stack = Self {
            button,
            icon,
            popup,
            folder_path,
            entries,
            view_mode,
            monitor: None,
            max_items: 20,
        };
        
        // Load initial entries
        stack.refresh_entries();
        
        // Setup click handler
        stack.setup_click_handler();
        
        // Start monitoring
        stack.start_monitoring();
        
        stack
    }
    
    /// Create a stack for ~/Downloads
    pub fn downloads(icon_size: u32) -> Self {
        let downloads = glib::user_special_dir(glib::UserDirectory::Downloads)
            .unwrap_or_else(|| PathBuf::from(std::env::var("HOME").unwrap_or_default()).join("Downloads"));
        Self::new(downloads, icon_size)
    }
    
    /// Get the widget
    pub fn widget(&self) -> &Button {
        &self.button
    }
    
    /// Refresh entries from the folder
    pub fn refresh_entries(&self) {
        let mut entries = Vec::new();
        
        if let Ok(read_dir) = std::fs::read_dir(&self.folder_path) {
            for entry in read_dir.filter_map(|e| e.ok()) {
                let path = entry.path();
                let name = path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                
                // Skip hidden files
                if name.starts_with('.') {
                    continue;
                }
                
                let is_directory = path.is_dir();
                let icon_name = Self::get_icon_for_file(&path, is_directory);
                
                // Get modification time
                let modified = entry.metadata().ok()
                    .and_then(|m| m.modified().ok())
                    .and_then(|t| {
                        let duration = t.duration_since(std::time::UNIX_EPOCH).ok()?;
                        glib::DateTime::from_unix_local(duration.as_secs() as i64).ok()
                    });
                
                entries.push(StackEntry {
                    name,
                    path,
                    icon_name,
                    is_directory,
                    modified,
                });
            }
        }
        
        // Sort by modification time (newest first)
        entries.sort_by(|a, b| b.modified.as_ref().map(|d| d.to_unix())
            .cmp(&a.modified.as_ref().map(|d| d.to_unix())));
        
        // Limit entries
        entries.truncate(self.max_items);
        
        *self.entries.borrow_mut() = entries;
        debug!("Stack refreshed with {} entries", self.entries.borrow().len());
    }
    
    /// Get appropriate icon for a file
    fn get_icon_for_file(path: &PathBuf, is_dir: bool) -> String {
        if is_dir {
            return "folder".to_string();
        }
        
        let extension = path.extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        
        match extension.as_str() {
            "pdf" => "application-pdf",
            "zip" | "tar" | "gz" | "xz" | "7z" | "rar" => "package-x-generic",
            "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" => "image-x-generic",
            "mp3" | "wav" | "flac" | "ogg" | "m4a" => "audio-x-generic",
            "mp4" | "mkv" | "avi" | "mov" | "webm" => "video-x-generic",
            "doc" | "docx" | "odt" => "x-office-document",
            "xls" | "xlsx" | "ods" => "x-office-spreadsheet",
            "ppt" | "pptx" | "odp" => "x-office-presentation",
            "txt" | "md" | "rst" => "text-x-generic",
            "rs" | "py" | "js" | "ts" | "c" | "cpp" | "h" | "java" => "text-x-script",
            "html" | "css" | "xml" | "json" => "text-html",
            "deb" | "rpm" | "appimage" => "application-x-executable",
            _ => "text-x-generic",
        }.to_string()
    }
    
    /// Create the popup widget
    fn create_popup() -> gtk::Popover {
        let popup = gtk::Popover::builder()
            .has_arrow(true)
            .css_classes(vec!["stack-popup"])
            .build();
        
        popup
    }
    
    /// Setup click handler to show popup
    fn setup_click_handler(&self) {
        let popup = self.popup.clone();
        let entries = Rc::clone(&self.entries);
        let view_mode = Rc::clone(&self.view_mode);
        let folder_path = self.folder_path.clone();
        
        self.button.connect_clicked(move |_| {
            // Rebuild popup content
            let content = Self::build_popup_content(&entries.borrow(), *view_mode.borrow(), &folder_path);
            popup.set_child(Some(&content));
            popup.popup();
        });
    }
    
    /// Build popup content based on view mode
    fn build_popup_content(entries: &[StackEntry], mode: StackViewMode, folder_path: &PathBuf) -> gtk::Widget {
        match mode {
            StackViewMode::Grid => Self::build_grid_view(entries, folder_path),
            StackViewMode::List => Self::build_list_view(entries, folder_path),
            StackViewMode::Fan => Self::build_grid_view(entries, folder_path), // Fan uses grid for now
        }
    }
    
    /// Build grid view
    fn build_grid_view(entries: &[StackEntry], folder_path: &PathBuf) -> gtk::Widget {
        let flow_box = gtk::FlowBox::builder()
            .orientation(Orientation::Horizontal)
            .max_children_per_line(4)
            .min_children_per_line(2)
            .selection_mode(gtk::SelectionMode::None)
            .homogeneous(true)
            .row_spacing(8)
            .column_spacing(8)
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();
        
        if entries.is_empty() {
            let label = Label::new(Some("Folder is empty"));
            label.add_css_class("stack-empty-label");
            flow_box.insert(&label, -1);
        } else {
            for entry in entries {
                let card = Self::create_file_card(entry);
                flow_box.insert(&card, -1);
            }
        }
        
        // Add "Open in Files" button at bottom
        let open_button = Button::builder()
            .label("Open in Files")
            .css_classes(vec!["stack-open-button"])
            .build();
        
        let path = folder_path.clone();
        open_button.connect_clicked(move |_| {
            if let Err(e) = std::process::Command::new("xdg-open")
                .arg(&path)
                .spawn() 
            {
                warn!("Failed to open folder: {}", e);
            }
        });
        
        let container = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(8)
            .build();
        
        let scroll = ScrolledWindow::builder()
            .min_content_height(200)
            .max_content_height(400)
            .min_content_width(300)
            .child(&flow_box)
            .build();
        
        container.append(&scroll);
        container.append(&open_button);
        
        container.upcast()
    }
    
    /// Build list view
    fn build_list_view(entries: &[StackEntry], folder_path: &PathBuf) -> gtk::Widget {
        let list_box = gtk::ListBox::builder()
            .selection_mode(gtk::SelectionMode::None)
            .css_classes(vec!["stack-list"])
            .build();
        
        for entry in entries {
            let row = Self::create_list_row(entry);
            list_box.append(&row);
        }
        
        let scroll = ScrolledWindow::builder()
            .min_content_height(200)
            .max_content_height(400)
            .min_content_width(250)
            .child(&list_box)
            .build();
        
        scroll.upcast()
    }
    
    /// Create a file card for grid view
    fn create_file_card(entry: &StackEntry) -> gtk::Widget {
        let card = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(4)
            .halign(gtk::Align::Center)
            .css_classes(vec!["stack-file-card"])
            .build();
        
        let icon = Image::from_icon_name(&entry.icon_name);
        icon.set_pixel_size(48);
        
        let name = entry.name.chars().take(15).collect::<String>();
        let label = Label::builder()
            .label(&name)
            .ellipsize(gtk::pango::EllipsizeMode::End)
            .max_width_chars(12)
            .css_classes(vec!["stack-file-name"])
            .build();
        
        card.append(&icon);
        card.append(&label);
        
        // Make clickable
        let button = Button::builder()
            .child(&card)
            .css_classes(vec!["stack-file-button"])
            .tooltip_text(&entry.name)
            .build();
        
        let path = entry.path.clone();
        button.connect_clicked(move |_| {
            info!("Opening file: {:?}", path);
            if let Err(e) = std::process::Command::new("xdg-open")
                .arg(&path)
                .spawn()
            {
                warn!("Failed to open file: {}", e);
            }
        });
        
        button.upcast()
    }
    
    /// Create a list row
    fn create_list_row(entry: &StackEntry) -> gtk::Widget {
        let row = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .margin_start(8)
            .margin_end(8)
            .margin_top(4)
            .margin_bottom(4)
            .build();
        
        let icon = Image::from_icon_name(&entry.icon_name);
        icon.set_pixel_size(24);
        
        let label = Label::builder()
            .label(&entry.name)
            .ellipsize(gtk::pango::EllipsizeMode::End)
            .xalign(0.0)
            .hexpand(true)
            .build();
        
        row.append(&icon);
        row.append(&label);
        
        let button = Button::builder()
            .child(&row)
            .css_classes(vec!["stack-list-row"])
            .build();
        
        let path = entry.path.clone();
        button.connect_clicked(move |_| {
            if let Err(e) = std::process::Command::new("xdg-open")
                .arg(&path)
                .spawn()
            {
                warn!("Failed to open file: {}", e);
            }
        });
        
        button.upcast()
    }
    
    /// Start monitoring the folder for changes
    fn start_monitoring(&mut self) {
        let file = gio::File::for_path(&self.folder_path);
        
        match file.monitor_directory(gio::FileMonitorFlags::NONE, gio::Cancellable::NONE) {
            Ok(monitor) => {
                let entries = Rc::clone(&self.entries);
                let folder_path = self.folder_path.clone();
                let max_items = self.max_items;
                
                monitor.connect_changed(move |_monitor, _file, _other, event| {
                    match event {
                        FileMonitorEvent::Created |
                        FileMonitorEvent::Deleted |
                        FileMonitorEvent::MovedIn |
                        FileMonitorEvent::MovedOut => {
                            debug!("Stack folder changed: {:?}", event);
                            // Refresh entries
                            Self::refresh_entries_static(&entries, &folder_path, max_items);
                        }
                        _ => {}
                    }
                });
                
                self.monitor = Some(monitor);
                info!("Stack monitoring started for {:?}", self.folder_path);
            }
            Err(e) => {
                warn!("Failed to monitor stack folder: {}", e);
            }
        }
    }
    
    /// Static refresh method for use in callbacks
    fn refresh_entries_static(
        entries: &Rc<RefCell<Vec<StackEntry>>>,
        folder_path: &PathBuf,
        max_items: usize,
    ) {
        let mut new_entries = Vec::new();
        
        if let Ok(read_dir) = std::fs::read_dir(folder_path) {
            for entry in read_dir.filter_map(|e| e.ok()) {
                let path = entry.path();
                let name = path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                
                if name.starts_with('.') {
                    continue;
                }
                
                let is_directory = path.is_dir();
                let icon_name = Self::get_icon_for_file(&path, is_directory);
                
                let modified = entry.metadata().ok()
                    .and_then(|m| m.modified().ok())
                    .and_then(|t| {
                        let duration = t.duration_since(std::time::UNIX_EPOCH).ok()?;
                        glib::DateTime::from_unix_local(duration.as_secs() as i64).ok()
                    });
                
                new_entries.push(StackEntry {
                    name,
                    path,
                    icon_name,
                    is_directory,
                    modified,
                });
            }
        }
        
        new_entries.sort_by(|a, b| b.modified.as_ref().map(|d| d.to_unix())
            .cmp(&a.modified.as_ref().map(|d| d.to_unix())));
        new_entries.truncate(max_items);
        
        *entries.borrow_mut() = new_entries;
    }
    
    /// Set the view mode
    pub fn set_view_mode(&self, mode: StackViewMode) {
        *self.view_mode.borrow_mut() = mode;
    }
}

/// CSS for stack popup
pub fn get_stack_css() -> &'static str {
    r#"
    .stack-popup {
        background: alpha(@window_bg_color, 0.95);
        border-radius: 12px;
    }
    
    .stack-file-button {
        background: transparent;
        border: none;
        padding: 8px;
        border-radius: 8px;
    }
    
    .stack-file-button:hover {
        background: alpha(@accent_bg_color, 0.3);
    }
    
    .stack-file-card {
        padding: 8px;
    }
    
    .stack-file-name {
        font-size: 11px;
    }
    
    .stack-list-row {
        background: transparent;
        border: none;
        border-radius: 6px;
    }
    
    .stack-list-row:hover {
        background: alpha(@accent_bg_color, 0.2);
    }
    
    .stack-open-button {
        margin: 8px;
    }
    
    .stack-empty-label {
        padding: 20px;
        color: alpha(@window_fg_color, 0.7);
    }
    "#
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_view_mode_default() {
        assert_eq!(StackViewMode::default(), StackViewMode::Fan);
    }
    
    #[test]
    fn test_icon_mapping() {
        assert_eq!(StackItem::get_icon_for_file(&PathBuf::from("test.pdf"), false), "application-pdf");
        assert_eq!(StackItem::get_icon_for_file(&PathBuf::from("test.png"), false), "image-x-generic");
        assert_eq!(StackItem::get_icon_for_file(&PathBuf::from("folder"), true), "folder");
    }
    
    #[test]
    fn test_stack_css() {
        let css = get_stack_css();
        assert!(css.contains(".stack-popup"));
        assert!(css.contains(".stack-file-button"));
    }
}
