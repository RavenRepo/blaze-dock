//! Progress ring widget using Cairo drawing
//!
//! Circular progress indicator for background operations (downloads, updates, etc.)

use gtk::prelude::*;
use gtk::glib;
use gtk::DrawingArea;
use std::f64::consts::PI;
use std::cell::Cell;
use std::rc::Rc;
use log::debug;

/// Progress ring widget
pub struct ProgressRing {
    drawing_area: DrawingArea,
    progress: Rc<Cell<f64>>,
    is_indeterminate: Rc<Cell<bool>>,
    animation_angle: Rc<Cell<f64>>,
}

impl ProgressRing {
    /// Create a new progress ring
    pub fn new(size: i32) -> Self {
        let drawing_area = DrawingArea::builder()
            .width_request(size)
            .height_request(size)
            .css_classes(vec!["progress-ring"])
            .build();

        let progress: Rc<Cell<f64>> = Rc::new(Cell::new(0.0_f64));
        let is_indeterminate: Rc<Cell<bool>> = Rc::new(Cell::new(false));
        let animation_angle: Rc<Cell<f64>> = Rc::new(Cell::new(0.0_f64));

        // Set up drawing function
        let progress_clone = Rc::clone(&progress);
        let is_indeterminate_clone = Rc::clone(&is_indeterminate);
        let animation_angle_clone = Rc::clone(&animation_angle);

        drawing_area.set_draw_func(move |_area, cr, width, height| {
            let size = width.min(height) as f64;
            let center_x = width as f64 / 2.0;
            let center_y = height as f64 / 2.0;
            let radius = (size / 2.0) - 4.0;
            let line_width = 3.0;

            // Background ring
            cr.set_line_width(line_width);
            cr.set_source_rgba(1.0, 1.0, 1.0, 0.2);
            cr.arc(center_x, center_y, radius, 0.0, 2.0 * PI);
            let _ = cr.stroke();

            // Progress arc
            if is_indeterminate_clone.get() {
                // Indeterminate: spinning arc
                let angle = animation_angle_clone.get();
                let arc_length = PI * 0.75;
                
                // Gradient effect for indeterminate
                cr.set_source_rgba(0.4, 0.8, 1.0, 1.0);
                cr.set_line_cap(cairo::LineCap::Round);
                cr.arc(
                    center_x,
                    center_y,
                    radius,
                    angle - PI / 2.0,
                    angle - PI / 2.0 + arc_length,
                );
                let _ = cr.stroke();
            } else {
                // Determinate: progress arc
                let progress_val = f64::clamp(progress_clone.get(), 0.0, 1.0);
                
                if progress_val > 0.0 {
                    // Progress color (blue gradient)
                    cr.set_source_rgba(0.4, 0.8, 1.0, 1.0);
                    cr.set_line_cap(cairo::LineCap::Round);
                    
                    // Draw from top (-PI/2) clockwise
                    let start_angle = -PI / 2.0;
                    let end_angle = start_angle + (2.0 * PI * progress_val);
                    
                    cr.arc(center_x, center_y, radius, start_angle, end_angle);
                    let _ = cr.stroke();
                    
                    // Add glow effect for high progress
                    if progress_val > 0.9 {
                        cr.set_source_rgba(0.4, 0.8, 1.0, 0.3);
                        cr.set_line_width(line_width + 4.0);
                        cr.arc(center_x, center_y, radius, start_angle, end_angle);
                        let _ = cr.stroke();
                    }
                }
            }
        });

        Self {
            drawing_area,
            progress,
            is_indeterminate,
            animation_angle,
        }
    }

    /// Get the widget
    pub fn widget(&self) -> &DrawingArea {
        &self.drawing_area
    }

    /// Set progress (0.0 - 1.0)
    pub fn set_progress(&self, value: f64) {
        self.progress.set(value.clamp(0.0, 1.0));
        self.is_indeterminate.set(false);
        self.drawing_area.queue_draw();
        debug!("Progress ring set to {:.0}%", value * 100.0);
    }

    /// Get current progress
    pub fn get_progress(&self) -> f64 {
        self.progress.get()
    }

    /// Set indeterminate mode (spinning)
    pub fn set_indeterminate(&self, indeterminate: bool) {
        self.is_indeterminate.set(indeterminate);
        
        if indeterminate {
            self.start_animation();
        }
        
        self.drawing_area.queue_draw();
    }

    /// Start spinning animation for indeterminate mode
    fn start_animation(&self) {
        let drawing_area = self.drawing_area.clone();
        let animation_angle = Rc::clone(&self.animation_angle);
        let is_indeterminate = Rc::clone(&self.is_indeterminate);

        glib::timeout_add_local(std::time::Duration::from_millis(16), move || {
            if !is_indeterminate.get() {
                return glib::ControlFlow::Break;
            }
            
            let current = animation_angle.get();
            animation_angle.set((current + 0.1) % (2.0 * PI));
            drawing_area.queue_draw();
            
            glib::ControlFlow::Continue
        });
    }

    /// Show the progress ring
    pub fn show(&self) {
        self.drawing_area.set_visible(true);
    }

    /// Hide the progress ring
    pub fn hide(&self) {
        self.drawing_area.set_visible(false);
        self.is_indeterminate.set(false);
    }
}

impl Default for ProgressRing {
    fn default() -> Self {
        Self::new(24)
    }
}

