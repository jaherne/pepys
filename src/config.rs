use anyhow::Result;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default = "default_placeholder_color")]
    pub placeholder_color: String,
}

fn default_placeholder_color() -> String {
    "red".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            placeholder_color: default_placeholder_color(),
        }
    }
}

impl Config {
    /// Load config from the standard config file location.
    /// Returns default config if file doesn't exist or can't be parsed.
    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;

        if !config_path.exists() {
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    /// Get the path to the config file.
    /// Uses ~/.pepys/config.toml
    pub fn get_config_path() -> Result<PathBuf> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Failed to determine home directory"))?;

        let config_dir = home_dir.join(".pepys");
        fs::create_dir_all(&config_dir)?;
        Ok(config_dir.join("config.toml"))
    }

    /// Convert the placeholder color to a bash ANSI escape sequence.
    pub fn bash_color_code(&self) -> &str {
        match self.placeholder_color.to_lowercase().as_str() {
            "black" => "0;30",
            "red" => "0;31",
            "green" => "0;32",
            "yellow" => "0;33",
            "blue" => "0;34",
            "magenta" => "0;35",
            "cyan" => "0;36",
            "white" => "0;37",
            _ => "0;31", // default to red for unknown colors
        }
    }

    /// Convert the placeholder color to a zsh color name.
    /// Zsh supports these color names natively.
    pub fn zsh_color_name(&self) -> &str {
        match self.placeholder_color.to_lowercase().as_str() {
            "black" | "red" | "green" | "yellow" | "blue" | "magenta" | "cyan" | "white" => {
                // Return the lowercased version for valid colors
                match self.placeholder_color.to_lowercase().as_str() {
                    "black" => "black",
                    "red" => "red",
                    "green" => "green",
                    "yellow" => "yellow",
                    "blue" => "blue",
                    "magenta" => "magenta",
                    "cyan" => "cyan",
                    "white" => "white",
                    _ => "red",
                }
            }
            _ => "red", // default to red for unknown colors
        }
    }
}
