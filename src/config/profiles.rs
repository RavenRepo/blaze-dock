//! Profile system for multiple dock configurations
//!
//! Allows users to switch between different dock setups (work, gaming, presentation, etc.)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use log::{info, debug, error, warn};
use directories::ProjectDirs;

use crate::config::settings::{Settings, DockPosition};

/// Profile metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileMeta {
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub created_at: String,
    pub last_used: Option<String>,
}

/// Complete profile with settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub meta: ProfileMeta,
    pub settings: Settings,
}

/// Profile manager for handling multiple configurations
#[derive(Clone)]
pub struct ProfileManager {
    profiles_dir: PathBuf,
    current_profile: String,
    profiles: HashMap<String, Profile>,
}

impl ProfileManager {
    /// Create a new profile manager
    pub fn new() -> Self {
        let profiles_dir = Self::get_profiles_dir();
        
        // Ensure profiles directory exists
        if !profiles_dir.exists() {
            if let Err(e) = fs::create_dir_all(&profiles_dir) {
                error!("Failed to create profiles directory: {}", e);
            }
        }

        let mut manager = Self {
            profiles_dir,
            current_profile: "default".to_string(),
            profiles: HashMap::new(),
        };

        manager.load_all_profiles();
        manager.ensure_default_profile();
        
        manager
    }

    /// Get profiles directory path
    fn get_profiles_dir() -> PathBuf {
        if let Some(proj_dirs) = ProjectDirs::from("com", "blazedock", "blazedock") {
            proj_dirs.config_dir().join("profiles")
        } else {
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("blazedock")
                .join("profiles")
        }
    }

    /// Load all profiles from disk
    fn load_all_profiles(&mut self) {
        if let Ok(entries) = fs::read_dir(&self.profiles_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "toml") {
                    if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                        if let Ok(profile) = self.load_profile_from_file(&path) {
                            self.profiles.insert(name.to_string(), profile);
                            debug!("Loaded profile: {}", name);
                        }
                    }
                }
            }
        }
        
        info!("Loaded {} profiles", self.profiles.len());
    }

    /// Load a single profile from file
    fn load_profile_from_file(&self, path: &PathBuf) -> Result<Profile, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let profile: Profile = toml::from_str(&content)?;
        Ok(profile)
    }

    /// Ensure default profile exists
    fn ensure_default_profile(&mut self) {
        if !self.profiles.contains_key("default") {
            let default_profile = Profile {
                meta: ProfileMeta {
                    name: "Default".to_string(),
                    description: Some("Default dock configuration".to_string()),
                    icon: Some("user-home".to_string()),
                    created_at: chrono_lite_now(),
                    last_used: Some(chrono_lite_now()),
                },
                settings: Settings::default(),
            };
            
            self.profiles.insert("default".to_string(), default_profile.clone());
            let _ = self.save_profile("default", &default_profile);
            info!("Created default profile");
        }
    }

    /// Save a profile to disk
    pub fn save_profile(&self, name: &str, profile: &Profile) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.profiles_dir.join(format!("{}.toml", name));
        let content = toml::to_string_pretty(profile)?;
        fs::write(&path, content)?;
        info!("Saved profile: {}", name);
        Ok(())
    }

    /// Create a new profile
    pub fn create_profile(&mut self, name: &str, description: Option<&str>, base_settings: Option<Settings>) -> Result<(), String> {
        if self.profiles.contains_key(name) {
            return Err(format!("Profile '{}' already exists", name));
        }

        let profile = Profile {
            meta: ProfileMeta {
                name: name.to_string(),
                description: description.map(|s| s.to_string()),
                icon: None,
                created_at: chrono_lite_now(),
                last_used: None,
            },
            settings: base_settings.unwrap_or_default(),
        };

        if let Err(e) = self.save_profile(name, &profile) {
            return Err(format!("Failed to save profile: {}", e));
        }

        self.profiles.insert(name.to_string(), profile);
        info!("Created new profile: {}", name);
        Ok(())
    }

    /// Delete a profile
    pub fn delete_profile(&mut self, name: &str) -> Result<(), String> {
        if name == "default" {
            return Err("Cannot delete default profile".to_string());
        }

        if !self.profiles.contains_key(name) {
            return Err(format!("Profile '{}' does not exist", name));
        }

        let path = self.profiles_dir.join(format!("{}.toml", name));
        if let Err(e) = fs::remove_file(&path) {
            warn!("Failed to remove profile file: {}", e);
        }

        self.profiles.remove(name);
        
        if self.current_profile == name {
            self.current_profile = "default".to_string();
        }
        
        info!("Deleted profile: {}", name);
        Ok(())
    }

    /// Switch to a different profile
    pub fn switch_profile(&mut self, name: &str) -> Result<Settings, String> {
        if !self.profiles.contains_key(name) {
            return Err(format!("Profile '{}' does not exist", name));
        }

        self.current_profile = name.to_string();
        
        // Update last_used timestamp
        if let Some(profile) = self.profiles.get_mut(name) {
            profile.meta.last_used = Some(chrono_lite_now());
        }
        
        // Save profile (after releasing the mutable borrow)
        if let Some(profile) = self.profiles.get(name) {
            let _ = self.save_profile(name, profile);
        }

        let settings = self.profiles.get(name)
            .map(|p| p.settings.clone())
            .unwrap_or_default();
        
        info!("Switched to profile: {}", name);
        Ok(settings)
    }

    /// Get current profile name
    pub fn current_profile_name(&self) -> &str {
        &self.current_profile
    }

    /// Get current profile settings
    pub fn current_settings(&self) -> Settings {
        self.profiles.get(&self.current_profile)
            .map(|p| p.settings.clone())
            .unwrap_or_default()
    }

    /// Update settings in current profile
    pub fn update_current_settings(&mut self, settings: Settings) -> Result<(), String> {
        let current_profile_name = self.current_profile.clone();
        
        if let Some(profile) = self.profiles.get_mut(&current_profile_name) {
            profile.settings = settings;
        } else {
            return Err("No current profile".to_string());
        }
        
        // Save profile (after releasing the mutable borrow)
        if let Some(profile) = self.profiles.get(&current_profile_name) {
            self.save_profile(&current_profile_name, profile)
                .map_err(|e| e.to_string())?;
        }
        
        Ok(())
    }

    /// List all available profiles
    pub fn list_profiles(&self) -> Vec<(&str, &ProfileMeta)> {
        self.profiles.iter()
            .map(|(name, profile)| (name.as_str(), &profile.meta))
            .collect()
    }

    /// Get profile by name
    pub fn get_profile(&self, name: &str) -> Option<&Profile> {
        self.profiles.get(name)
    }

    /// Duplicate an existing profile
    pub fn duplicate_profile(&mut self, source: &str, new_name: &str) -> Result<(), String> {
        let source_profile = self.profiles.get(source)
            .ok_or_else(|| format!("Source profile '{}' does not exist", source))?
            .clone();

        if self.profiles.contains_key(new_name) {
            return Err(format!("Profile '{}' already exists", new_name));
        }

        let new_profile = Profile {
            meta: ProfileMeta {
                name: new_name.to_string(),
                description: source_profile.meta.description.map(|d| format!("{} (copy)", d)),
                icon: source_profile.meta.icon,
                created_at: chrono_lite_now(),
                last_used: None,
            },
            settings: source_profile.settings,
        };

        self.save_profile(new_name, &new_profile)
            .map_err(|e| e.to_string())?;
        self.profiles.insert(new_name.to_string(), new_profile);
        
        info!("Duplicated profile '{}' to '{}'", source, new_name);
        Ok(())
    }

    /// Create preset profiles
    pub fn create_presets(&mut self) {
        // Work profile - minimal distractions
        if !self.profiles.contains_key("work") {
            let mut work_settings = Settings::default();
            work_settings.icon_size = 40;
            work_settings.auto_hide = true;
            work_settings.hover_zoom = false;
            
            let _ = self.create_profile("work", Some("Minimal dock for focused work"), Some(work_settings));
        }

        // Gaming profile - out of the way
        if !self.profiles.contains_key("gaming") {
            let mut gaming_settings = Settings::default();
            gaming_settings.auto_hide = true;
            gaming_settings.opacity = 0.7;
            gaming_settings.position = DockPosition::Left;
            
            let _ = self.create_profile("gaming", Some("Auto-hiding dock for gaming"), Some(gaming_settings));
        }

        // Presentation profile - large icons, no distractions
        if !self.profiles.contains_key("presentation") {
            let mut presentation_settings = Settings::default();
            presentation_settings.icon_size = 64;
            presentation_settings.auto_hide = true;
            presentation_settings.hover_zoom = true;
            presentation_settings.hover_zoom_scale = 1.8;
            
            let _ = self.create_profile("presentation", Some("Large icons for presentations"), Some(presentation_settings));
        }

        info!("Created preset profiles");
    }
}

impl Default for ProfileManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple timestamp function (avoids chrono dependency)
fn chrono_lite_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    
    format!("{}", duration.as_secs())
}

