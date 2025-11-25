//! Badge system for dock items
//!
//! Supports notification counts, progress indicators, and custom badges.

use gtk::prelude::*;
use gtk::{Box as GtkBox, Label};
use log::debug;

/// Badge types
#[derive(Debug, Clone)]
pub enum BadgeType {
    /// Notification count badge
    Count(u32),
    /// Progress indicator (0.0 - 1.0)
    Progress(f64),
    /// Attention/urgent indicator
    Attention,
    /// Custom icon badge
    Custom { icon: String },
}

/// Badge position
#[derive(Debug, Clone, Copy)]
pub enum BadgePosition {
    TopRight,
    BottomRight,
    TopLeft,
    BottomLeft,
    Center,
}

/// Badge widget
pub struct Badge {
    container: GtkBox,
    badge_type: BadgeType,
    position: BadgePosition,
}

impl Badge {
    /// Create a new badge
    pub fn new(badge_type: BadgeType, position: BadgePosition) -> Self {
        let container = GtkBox::builder()
            .css_classes(vec!["badge", "badge-hidden"])
            .build();

        let mut badge = Self {
            container,
            badge_type,
            position,
        };

        badge.update_display();
        badge
    }

    /// Get the widget
    pub fn widget(&self) -> &GtkBox {
        &self.container
    }

    /// Update badge type
    pub fn set_type(&mut self, badge_type: BadgeType) {
        self.badge_type = badge_type;
        self.update_display();
    }

    /// Update the visual display
    fn update_display(&mut self) {
        // Clear existing content
        while let Some(child) = self.container.first_child() {
            self.container.remove(&child);
        }

        match &self.badge_type {
            BadgeType::Count(count) => {
                if *count > 0 {
                    self.container.remove_css_class("badge-hidden");
                    self.container.add_css_class("badge-count");
                    
                    let text = if *count > 99 {
                        "99+".to_string()
                    } else {
                        format!("{}", count)
                    };
                    let label = Label::new(Some(&text));
                    label.add_css_class("badge-label");
                    self.container.append(&label);
                } else {
                    self.container.add_css_class("badge-hidden");
                }
            }
            BadgeType::Progress(progress) => {
                self.container.remove_css_class("badge-hidden");
                self.container.add_css_class("badge-progress");
                // Progress ring will be drawn via CSS/Cairo
                debug!("Progress badge: {:.0}%", progress * 100.0);
            }
            BadgeType::Attention => {
                self.container.remove_css_class("badge-hidden");
                self.container.add_css_class("badge-attention");
            }
            BadgeType::Custom { icon: _ } => {
                self.container.remove_css_class("badge-hidden");
                self.container.add_css_class("badge-custom");
                // TODO: Add icon
            }
        }

        // Set position class
        let position_class = match self.position {
            BadgePosition::TopRight => "badge-top-right",
            BadgePosition::BottomRight => "badge-bottom-right",
            BadgePosition::TopLeft => "badge-top-left",
            BadgePosition::BottomLeft => "badge-bottom-left",
            BadgePosition::Center => "badge-center",
        };
        self.container.add_css_class(position_class);
    }
}

