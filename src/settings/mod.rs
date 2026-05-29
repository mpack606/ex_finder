use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub quick_access_paths: Vec<PathBuf>,
    pub last_directory: Option<PathBuf>,
    pub window_width: u32,
    pub window_height: u32,
}

impl Default for Settings {
    fn default() -> Self {
        let mut quick_access_paths = Vec::new();
        if let Some(home) = dirs::home_dir() {
            quick_access_paths.push(home.clone());
            
            let desktop = home.join("Desktop");
            if desktop.exists() {
                quick_access_paths.push(desktop);
            }
            
            let documents = home.join("Documents");
            if documents.exists() {
                quick_access_paths.push(documents);
            }
            
            let downloads = home.join("Downloads");
            if downloads.exists() {
                quick_access_paths.push(downloads);
            }
        }
        
        let apps = PathBuf::from("/Applications");
        if apps.exists() {
            quick_access_paths.push(apps);
        }

        Self {
            quick_access_paths,
            last_directory: dirs::home_dir(),
            window_width: 1024,
            window_height: 768,
        }
    }
}

pub fn settings_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".ex_finder.toml"))
}

pub fn load_settings() -> Settings {
    if let Some(path) = settings_path() {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(settings) = toml::from_str::<Settings>(&content) {
                    return settings;
                }
            }
        }
    }
    // If loading fails or file does not exist, save and return defaults
    let default_settings = Settings::default();
    let _ = save_settings(&default_settings);
    default_settings
}

pub fn save_settings(settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(path) = settings_path() {
        let content = toml::to_string_pretty(settings)?;
        fs::write(path, content)?;
    }
    Ok(())
}
