//! Magnification controller
//!
//! Implements macOS-style cosine-based magnification with smooth animations.

use log::debug;

/// Magnification controller for dock items
pub struct MagnificationController {
    max_scale: f64,
    range_items: usize,
    animation_duration_ms: u32,
    current_hover: Option<usize>,
}

impl MagnificationController {
    /// Create a new magnification controller
    pub fn new(max_scale: f64, range_items: usize) -> Self {
        Self {
            max_scale,
            range_items,
            animation_duration_ms: 200,
            current_hover: None,
        }
    }

    /// Calculate magnification scale for an item based on distance from hover
    ///
    /// Uses cosine interpolation for smooth falloff
    pub fn calculate_scale(&self, item_index: usize, hover_index: Option<usize>) -> f64 {
        let hover_index = match hover_index {
            Some(idx) => idx,
            None => return 1.0, // No hover, no magnification
        };

        let distance = (item_index as i32 - hover_index as i32).abs() as usize;
        
        if distance > self.range_items {
            return 1.0; // Out of range
        }

        if distance == 0 {
            return self.max_scale; // Hovered item gets full magnification
        }

        // Cosine interpolation for smooth falloff
        let normalized = distance as f64 / self.range_items as f64;
        let cosine_factor = (1.0 + (std::f64::consts::PI * normalized).cos()) / 2.0;
        
        // Scale from 1.0 to max_scale based on cosine
        1.0 + (self.max_scale - 1.0) * cosine_factor
    }

    /// Set the currently hovered item index
    pub fn set_hover(&mut self, index: Option<usize>) {
        if self.current_hover != index {
            debug!("Magnification hover changed: {:?} -> {:?}", self.current_hover, index);
            self.current_hover = index;
        }
    }

    /// Get current hover index
    pub fn hover_index(&self) -> Option<usize> {
        self.current_hover
    }

    /// Get animation duration in milliseconds
    pub fn animation_duration_ms(&self) -> u32 {
        self.animation_duration_ms
    }
}

impl Default for MagnificationController {
    fn default() -> Self {
        Self::new(1.5, 2) // 150% max scale, affect 2 neighbors
    }
}

