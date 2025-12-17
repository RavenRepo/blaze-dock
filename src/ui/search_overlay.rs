//! Type-to-search overlay for filtering dock items
//!
//! Provides quick filtering by typing app names.

use gtk::prelude::*;
use gtk::{Box as GtkBox, Entry, Label, ListBox, ListBoxRow, Overlay};
use gtk::glib;
use log::debug;
use std::cell::RefCell;
use std::rc::Rc;

/// Search result item
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub app_id: String,
    pub name: String,
    pub icon_name: String,
    pub command: String,
    pub score: u32,
}

/// Search overlay widget
pub struct SearchOverlay {
    overlay: Overlay,
    search_box: GtkBox,
    entry: Entry,
    results_list: ListBox,
    visible: Rc<RefCell<bool>>,
    results: Rc<RefCell<Vec<SearchResult>>>,
    on_select: Rc<RefCell<Option<Box<dyn Fn(&SearchResult)>>>>,
}

impl SearchOverlay {
    /// Create a new search overlay
    pub fn new() -> Self {
        let overlay = Overlay::new();
        
        // Search box container
        let search_box = GtkBox::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(8)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Start)
            .margin_top(50)
            .width_request(400)
            .css_classes(vec!["search-overlay"])
            .visible(false)
            .build();

        // Search entry
        let entry = Entry::builder()
            .placeholder_text("Type to search apps...")
            .css_classes(vec!["search-entry"])
            .build();

        // Results list
        let results_list = ListBox::builder()
            .css_classes(vec!["search-results"])
            .selection_mode(gtk::SelectionMode::Single)
            .build();

        search_box.append(&entry);
        search_box.append(&results_list);

        let visible = Rc::new(RefCell::new(false));
        let results = Rc::new(RefCell::new(Vec::new()));
        let on_select: Rc<RefCell<Option<Box<dyn Fn(&SearchResult)>>>> = Rc::new(RefCell::new(None));

        let search_overlay = Self {
            overlay,
            search_box,
            entry,
            results_list,
            visible,
            results,
            on_select,
        };

        search_overlay.setup_signals();
        search_overlay
    }

    /// Setup event handlers
    fn setup_signals(&self) {
        let results_list = self.results_list.clone();
        let results = Rc::clone(&self.results);
        let on_select = Rc::clone(&self.on_select);
        let visible = Rc::clone(&self.visible);
        let search_box = self.search_box.clone();

        // Handle text changes
        self.entry.connect_changed(move |entry| {
            let query = entry.text().to_string();
            debug!("Search query: {}", query);
            
            // Clear previous results
            while let Some(row) = results_list.first_child() {
                results_list.remove(&row);
            }
            
            if query.is_empty() {
                return;
            }
            
            // Filter results (this would be populated by set_apps)
            let results_guard = results.borrow();
            let query_lower = query.to_lowercase();
            
            let mut filtered: Vec<_> = results_guard
                .iter()
                .filter(|r| r.name.to_lowercase().contains(&query_lower))
                .cloned()
                .collect();
            
            // Sort by relevance (starts with > contains)
            filtered.sort_by(|a, b| {
                let a_starts = a.name.to_lowercase().starts_with(&query_lower);
                let b_starts = b.name.to_lowercase().starts_with(&query_lower);
                b_starts.cmp(&a_starts).then_with(|| a.name.cmp(&b.name))
            });
            
            // Show top results
            for result in filtered.iter().take(8) {
                let row = Self::create_result_row(result);
                results_list.append(&row);
            }
        });

        // Handle selection
        let on_select_clone = Rc::clone(&self.on_select);
        let results_clone = Rc::clone(&self.results);
        let entry = self.entry.clone();
        
        self.results_list.connect_row_activated(move |_list, row| {
            let idx = row.index() as usize;
            let results_guard = results_clone.borrow();
            let query = entry.text().to_string().to_lowercase();
            
            let filtered: Vec<_> = results_guard
                .iter()
                .filter(|r| r.name.to_lowercase().contains(&query))
                .collect();
            
            if let Some(result) = filtered.get(idx) {
                if let Some(callback) = on_select_clone.borrow().as_ref() {
                    callback(result);
                }
            }
        });

        // Handle Escape key
        let visible_clone = Rc::clone(&self.visible);
        let search_box_clone = self.search_box.clone();
        
        let key_controller = gtk::EventControllerKey::new();
        key_controller.connect_key_pressed(move |_, key, _, _| {
            if key == gtk::gdk::Key::Escape {
                *visible_clone.borrow_mut() = false;
                search_box_clone.set_visible(false);
                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        });
        self.entry.add_controller(key_controller);
    }

    /// Create a result row widget
    fn create_result_row(result: &SearchResult) -> ListBoxRow {
        let row = ListBoxRow::builder()
            .css_classes(vec!["search-result-row"])
            .build();

        let hbox = GtkBox::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(12)
            .margin_start(12)
            .margin_end(12)
            .margin_top(8)
            .margin_bottom(8)
            .build();

        // Icon
        let icon = gtk::Image::from_icon_name(&result.icon_name);
        icon.set_pixel_size(32);
        icon.add_css_class("search-result-icon");

        // Name
        let label = Label::new(Some(&result.name));
        label.add_css_class("search-result-name");
        label.set_halign(gtk::Align::Start);
        label.set_hexpand(true);

        hbox.append(&icon);
        hbox.append(&label);
        row.set_child(Some(&hbox));

        row
    }

    /// Get the overlay widget
    pub fn widget(&self) -> &Overlay {
        &self.overlay
    }

    /// Set the main content widget
    pub fn set_child(&self, child: &impl IsA<gtk::Widget>) {
        self.overlay.set_child(Some(child));
        self.overlay.add_overlay(&self.search_box);
    }

    /// Set available apps for searching
    pub fn set_apps(&self, apps: Vec<SearchResult>) {
        *self.results.borrow_mut() = apps;
    }

    /// Register selection callback
    pub fn on_select<F>(&self, callback: F)
    where
        F: Fn(&SearchResult) + 'static,
    {
        *self.on_select.borrow_mut() = Some(Box::new(callback));
    }

    /// Show the search overlay
    pub fn show(&self) {
        *self.visible.borrow_mut() = true;
        self.search_box.set_visible(true);
        self.entry.set_text("");
        self.entry.grab_focus();
        debug!("Search overlay shown");
    }

    /// Hide the search overlay
    pub fn hide(&self) {
        *self.visible.borrow_mut() = false;
        self.search_box.set_visible(false);
        debug!("Search overlay hidden");
    }

    /// Check if visible
    pub fn is_visible(&self) -> bool {
        *self.visible.borrow()
    }

    /// Toggle visibility
    pub fn toggle(&self) {
        if self.is_visible() {
            self.hide();
        } else {
            self.show();
        }
    }
}

impl Default for SearchOverlay {
    fn default() -> Self {
        Self::new()
    }
}

