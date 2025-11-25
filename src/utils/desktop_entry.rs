//! Desktop entry parser
//!
//! Parses .desktop files to extract application information
//! for automatic app discovery and icon resolution.

use anyhow::{Context, Result};
use log::debug;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Standard application directories to search for .desktop files
pub const APP_DIRS: &[&str] = &[
    "/usr/share/applications",
    "/usr/local/share/applications",
];

/// User application directory (relative to home)
pub const USER_APP_DIR: &str = ".local/share/applications";

/// Represents a parsed .desktop file
#[derive(Debug, Clone)]
pub struct DesktopEntry {
    /// Full path to the .desktop file
    pub path: PathBuf,
    /// Application name
    pub name: Option<String>,
    /// Generic name (e.g., "Web Browser")
    pub generic_name: Option<String>,
    /// Comment/description
    pub comment: Option<String>,
    /// Icon name or path
    pub icon: Option<String>,
    /// Exec command
    pub exec: Option<String>,
    /// Whether this is a terminal application
    pub terminal: bool,
    /// Categories (e.g., "Network;WebBrowser")
    pub categories: Vec<String>,
    /// Whether the entry should be hidden
    pub no_display: bool,
    /// All key-value pairs from [Desktop Entry]
    pub fields: HashMap<String, String>,
}

impl DesktopEntry {
    /// Parse a .desktop file
    pub fn parse<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)
            .context(format!("Failed to read desktop file: {:?}", path))?;

        Self::parse_content(path.to_path_buf(), &content)
    }

    /// Parse desktop file content
    fn parse_content(path: PathBuf, content: &str) -> Result<Self> {
        let mut fields = HashMap::new();
        let mut in_desktop_entry = false;

        for line in content.lines() {
            let line = line.trim();
            
            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Check for section headers
            if line.starts_with('[') && line.ends_with(']') {
                in_desktop_entry = line == "[Desktop Entry]";
                continue;
            }

            // Only parse [Desktop Entry] section
            if !in_desktop_entry {
                continue;
            }

            // Parse key=value pairs
            if let Some((key, value)) = line.split_once('=') {
                fields.insert(key.trim().to_string(), value.trim().to_string());
            }
        }

        let categories = fields
            .get("Categories")
            .map(|c| c.split(';').filter(|s| !s.is_empty()).map(String::from).collect())
            .unwrap_or_default();

        Ok(Self {
            path,
            name: fields.get("Name").cloned(),
            generic_name: fields.get("GenericName").cloned(),
            comment: fields.get("Comment").cloned(),
            icon: fields.get("Icon").cloned(),
            exec: fields.get("Exec").cloned(),
            terminal: fields.get("Terminal").map(|v| v == "true").unwrap_or(false),
            categories,
            no_display: fields.get("NoDisplay").map(|v| v == "true").unwrap_or(false),
            fields,
        })
    }

    /// Get the exec command with field codes stripped
    ///
    /// Desktop files can contain field codes like:
    /// - %u - Single URL
    /// - %U - List of URLs
    /// - %f - Single file
    /// - %F - List of files
    /// - %i - Icon field
    /// - %c - Translated name
    /// - %k - Desktop file path
    pub fn exec_command(&self) -> Option<String> {
        self.exec.as_ref().map(|exec| {
            // Remove field codes
            let stripped = exec
                .replace("%u", "")
                .replace("%U", "")
                .replace("%f", "")
                .replace("%F", "")
                .replace("%i", "")
                .replace("%c", "")
                .replace("%k", "")
                .replace("%%", "%");
            
            // Clean up multiple spaces
            stripped.split_whitespace().collect::<Vec<_>>().join(" ")
        })
    }

    /// Check if this is a valid, visible application entry
    pub fn is_visible_app(&self) -> bool {
        !self.no_display 
            && self.name.is_some() 
            && self.exec.is_some()
            && self.fields.get("Type").map(|t| t == "Application").unwrap_or(false)
    }
}

/// Discover all installed applications
pub fn discover_applications() -> Vec<DesktopEntry> {
    let mut entries = Vec::new();

    // Search system directories
    for dir in APP_DIRS {
        if let Ok(read_dir) = fs::read_dir(dir) {
            for entry in read_dir.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.extension().map(|e| e == "desktop").unwrap_or(false) {
                    if let Ok(desktop) = DesktopEntry::parse(&path) {
                        if desktop.is_visible_app() {
                            debug!("Discovered app: {:?}", desktop.name);
                            entries.push(desktop);
                        }
                    }
                }
            }
        }
    }

    // Search user directory
    if let Some(home) = dirs::home_dir() {
        let user_apps = home.join(USER_APP_DIR);
        if let Ok(read_dir) = fs::read_dir(&user_apps) {
            for entry in read_dir.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.extension().map(|e| e == "desktop").unwrap_or(false) {
                    if let Ok(desktop) = DesktopEntry::parse(&path) {
                        if desktop.is_visible_app() {
                            debug!("Discovered user app: {:?}", desktop.name);
                            entries.push(desktop);
                        }
                    }
                }
            }
        }
    }

    // Sort by name
    entries.sort_by(|a, b| {
        a.name.as_deref().unwrap_or("")
            .cmp(b.name.as_deref().unwrap_or(""))
    });

    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_desktop_content() {
        let content = r#"
[Desktop Entry]
Type=Application
Name=Firefox
GenericName=Web Browser
Icon=firefox
Exec=firefox %u
Categories=Network;WebBrowser;
"#;

        let entry = DesktopEntry::parse_content(PathBuf::from("test.desktop"), content).unwrap();
        
        assert_eq!(entry.name, Some("Firefox".to_string()));
        assert_eq!(entry.icon, Some("firefox".to_string()));
        assert_eq!(entry.exec_command(), Some("firefox".to_string()));
        assert!(entry.categories.contains(&"Network".to_string()));
    }

    #[test]
    fn test_strip_field_codes() {
        let content = r#"
[Desktop Entry]
Type=Application
Name=Test
Exec=myapp --url %u --files %F
"#;

        let entry = DesktopEntry::parse_content(PathBuf::from("test.desktop"), content).unwrap();
        assert_eq!(entry.exec_command(), Some("myapp --url --files".to_string()));
    }
}

