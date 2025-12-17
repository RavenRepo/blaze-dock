//! Recent files service
//!
//! Tracks recently accessed files from the system.

use log::{info, debug};
use std::sync::{Arc, Mutex};
use std::path::PathBuf;

/// Recent file information
#[derive(Debug, Clone)]
pub struct RecentFile {
    pub name: String,
    pub path: PathBuf,
    pub timestamp: u64,
}

/// Recent files service
pub struct RecentFilesService {
    files: Arc<Mutex<Vec<RecentFile>>>,
}

impl RecentFilesService {
    /// Create a new recent files service
    pub fn new() -> Self {
        Self {
            files: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Refresh the list of recent files
    pub fn refresh(&self) {
        debug!("Refreshing recent files...");
        // In a real implementation, we would parse ~/.local/share/recently-used.xbel
        // or use GtkRecentManager
    }

    /// Get recent files
    pub fn get_recent_files(&self, limit: usize) -> Vec<RecentFile> {
        let files = self.files.lock().unwrap();
        files.iter().take(limit).cloned().collect()
    }
}

impl Default for RecentFilesService {
    fn default() -> Self {
        Self::new()
    }
}

