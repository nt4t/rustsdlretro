/// Configuration file system for rustsdlretro.
/// Loads settings from a JSON config file to select video backend and window parameters.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Video backend type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Renderer {
    Fbdev,
    Minifb,
}

impl Default for Renderer {
    fn default() -> Self {
        Renderer::Fbdev
    }
}

/// Window configuration for minifb renderer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    /// Window width in pixels
    pub width: u32,
    /// Window height in pixels
    pub height: u32,
    /// Integer scale factor (X1, X2, X3, X4)
    pub scale: u32,
    /// Whether to use a borderless window
    pub borderless: bool,
    /// Window title
    pub title: String,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: 640,
            height: 480,
            scale: 2,
            borderless: false,
            title: "rustsdlretro".to_string(),
        }
    }
}

/// Input configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputConfig {
    /// Input device path
    pub device: String,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            device: "/dev/input/event0".to_string(),
        }
    }
}

/// Full application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Video renderer to use
    #[serde(default)]
    pub renderer: Renderer,
    /// Window configuration (used when renderer = "minifb")
    #[serde(default)]
    pub window: WindowConfig,
    /// Input configuration
    #[serde(default)]
    pub input: InputConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            renderer: Renderer::default(),
            window: WindowConfig::default(),
            input: InputConfig::default(),
        }
    }
}

impl Config {
    /// Load config from the default path (~/.config/rustsdlretro/config.json)
    pub fn load_default() -> Self {
        let path = Self::default_path();
        Self::load(&path)
    }

    /// Load config from a specific path
    pub fn load(path: &Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(contents) => match serde_json::from_str(&contents) {
                Ok(config) => {
                    eprintln!("Config loaded from: {}", path.display());
                    config
                }
                Err(e) => {
                    eprintln!("Failed to parse config file {}: {}", path.display(), e);
                    Self::default()
                }
            },
            Err(_) => {
                eprintln!("Config file not found: {}, using defaults", path.display());
                Self::default()
            }
        }
    }

    /// Save config to the default path
    pub fn save_default(&self) -> std::io::Result<()> {
        let path = Self::default_path();
        self.save(&path)
    }

    /// Save config to a specific path
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let dir = path.parent().unwrap();
        std::fs::create_dir_all(dir)?;
        let contents = serde_json::to_string_pretty(self)?;
        std::fs::write(path, contents)
    }

    /// Get the default config path
    pub fn default_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_default();
        let path = PathBuf::from(home).join(".config").join("rustsdlretro").join("config.json");
        path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.renderer, Renderer::Fbdev);
        assert_eq!(config.window.width, 640);
        assert_eq!(config.window.height, 480);
        assert_eq!(config.window.scale, 2);
    }

    #[test]
    fn test_config_serialize_deserialize() {
        let config = Config {
            renderer: Renderer::Minifb,
            window: WindowConfig {
                width: 800,
                height: 600,
                scale: 3,
                borderless: true,
                title: "Test".to_string(),
            },
            input: InputConfig {
                device: "/dev/input/event1".to_string(),
            },
        };

        let json = serde_json::to_string_pretty(&config).unwrap();
        let loaded: Config = serde_json::from_str(&json).unwrap();

        assert_eq!(loaded.renderer, Renderer::Minifb);
        assert_eq!(loaded.window.width, 800);
        assert_eq!(loaded.window.height, 600);
        assert_eq!(loaded.window.scale, 3);
        assert_eq!(loaded.window.borderless, true);
        assert_eq!(loaded.window.title, "Test");
        assert_eq!(loaded.input.device, "/dev/input/event1");
    }
}
