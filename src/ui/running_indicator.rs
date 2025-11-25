//! Running application indicator
//!
//! Visual indicators showing which applications are currently running.

use gtk::prelude::*;
use gtk::{Box, Orientation};
use log::debug;

/// Running state for an application
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RunningState {
    /// Not running
    Stopped,
    /// Running (1 or more windows)
    Running { window_count: u8 },
    /// Running and focused
    Focused { window_count: u8 },
}

/// Visual indicator widget for running state
pub struct RunningIndicator {
    container: Box,
    state: RunningState,
}

impl RunningIndicator {
    /// Create a new running indicator
    pub fn new() -> Self {
        let container = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(4)
            .halign(gtk::Align::Center)
            .css_classes(vec!["running-indicator"])
            .build();

        Self {
            container,
            state: RunningState::Stopped,
        }
    }

    /// Get the widget
    pub fn widget(&self) -> &Box {
        &self.container
    }

    /// Update the running state
    pub fn set_state(&mut self, state: RunningState) {
        if self.state == state {
            return; // No change
        }

        self.state = state;
        self.update_display();
    }

    /// Get current state
    pub fn state(&self) -> RunningState {
        self.state
    }

    /// Update the visual display based on current state
    fn update_display(&self) {
        // Clear existing dots
        while let Some(child) = self.container.first_child() {
            self.container.remove(&child);
        }

        match self.state {
            RunningState::Stopped => {
                // No indicator when stopped
                self.container.add_css_class("stopped");
                self.container.remove_css_class("running");
                self.container.remove_css_class("focused");
            }
            RunningState::Running { window_count } => {
                self.container.remove_css_class("stopped");
                self.container.add_css_class("running");
                self.container.remove_css_class("focused");
                
                // Add dots for each window (max 3 visible)
                let visible_dots = window_count.min(3) as usize;
                for _ in 0..visible_dots {
                    let dot = Self::create_dot(false);
                    self.container.append(&dot);
                }
                
                // If more than 3 windows, show a number badge
                if window_count > 3 {
                    let label = gtk::Label::new(Some(&format!("+{}", window_count - 3)));
                    label.add_css_class("window-count-badge");
                    self.container.append(&label);
                }
            }
            RunningState::Focused { window_count } => {
                self.container.remove_css_class("stopped");
                self.container.add_css_class("running");
                self.container.add_css_class("focused");
                
                // Add dots with focused styling
                let visible_dots = window_count.min(3) as usize;
                for i in 0..visible_dots {
                    let is_focused = i == 0; // First dot is brighter when focused
                    let dot = Self::create_dot(is_focused);
                    self.container.append(&dot);
                }
                
                if window_count > 3 {
                    let label = gtk::Label::new(Some(&format!("+{}", window_count - 3)));
                    label.add_css_class("window-count-badge");
                    label.add_css_class("focused");
                    self.container.append(&label);
                }
            }
        }

        debug!("Updated running indicator: {:?}", self.state);
    }

    /// Create a single dot indicator
    fn create_dot(is_focused: bool) -> gtk::Widget {
        let dot = Box::builder()
            .width_request(6)
            .height_request(6)
            .css_classes(if is_focused {
                vec!["indicator-dot", "focused-dot"]
            } else {
                vec!["indicator-dot"]
            })
            .build();
        
        dot.upcast()
    }
}

impl Default for RunningIndicator {
    fn default() -> Self {
        Self::new()
    }
}

