//! CLI Configuration Management
//!
//! Reads config from .pkmai/config.toml in current directory or home directory.

use anyhow::Context;
use std::path::PathBuf;

/// Configuration structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Config {
    /// Database path (absolute or relative to config file location)
    pub database_path: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database_path: PathBuf::from("data.db"),
        }
    }
}

impl Config {
    /// Find config file by searching in order:
    /// 1. Current directory: .pkmai/config.toml
    /// 2. Home directory: ~/.pkmai/config.toml
    pub fn find() -> Option<Self> {
        // Try current directory: .pkmai/config.toml
        let current_config = std::path::Path::new(".pkmai/config.toml");
        if current_config.exists()
            && let Ok(content) = std::fs::read_to_string(current_config)
            && let Ok(config) = toml::from_str(&content)
        {
            return Some(config);
        }

        // Try home directory: ~/.pkmai/config.toml
        if let Some(home) = dirs::home_dir() {
            let mut path = home;
            path.push(".pkmai/config.toml");
            if path.exists()
                && let Ok(content) = std::fs::read_to_string(&path)
                && let Ok(config) = toml::from_str(&content)
            {
                return Some(config);
            }
        }

        None
    }

    /// Get the absolute database path, resolving relative paths from config location
    pub fn database_path(&self) -> PathBuf {
        if self.database_path.is_absolute() {
            self.database_path.clone()
        } else {
            // Relative path - resolve from .pkmai directory
            if let Some(home) = dirs::home_dir() {
                let mut path = home;
                path.push(".pkmai");
                path.push(&self.database_path);
                path
            } else {
                self.database_path.clone()
            }
        }
    }

    /// Initialize a new config in the current directory
    pub fn init_in_current_dir() -> anyhow::Result<Self> {
        let config_dir = std::path::Path::new(".pkmai");
        let config_file = config_dir.join("config.toml");

        // Create .pkmai directory if not exists
        if !config_dir.exists() {
            std::fs::create_dir_all(config_dir)
                .context("Failed to create .pkmai directory")?;
        }

        // Check if config already exists
        if config_file.exists() {
            anyhow::bail!("Config already exists at {}. Use --force to overwrite.", config_file.display());
        }

        let config = Self::default();
        let content = toml::to_string_pretty(&config)
            .context("Failed to serialize config")?;

        std::fs::write(&config_file, &content)
            .context("Failed to write config file")?;

        println!("Initialized PKM-AI in {}", std::fs::canonicalize(config_dir)?.display());
        println!("Config file: {}", config_file.display());
        println!("Database: {}", config.database_path().display());

        Ok(config)
    }

    /// Initialize a new config in the home directory (~/.pkmai/)
    pub fn init_in_home() -> anyhow::Result<Self> {
        let home = dirs::home_dir().context("Cannot find home directory")?;

        let config_dir = home.join(".pkmai");
        let config_file = config_dir.join("config.toml");

        // Create ~/.pkmai directory if not exists
        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir)
                .context("Failed to create .pkmai directory")?;
        }

        // Check if config already exists
        if config_file.exists() {
            anyhow::bail!("Config already exists at {}. Use --force to overwrite.", config_file.display());
        }

        let config = Self::default();
        let content = toml::to_string_pretty(&config)
            .context("Failed to serialize config")?;

        std::fs::write(&config_file, &content)
            .context("Failed to write config file")?;

        println!("Initialized PKM-AI in home directory");
        println!("Config file: {}", config_file.display());
        println!("Database: {}", config.database_path().display());

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.database_path, PathBuf::from("data.db"));
    }

    #[test]
    fn test_database_path_absolute() {
        let config = Config {
            database_path: PathBuf::from("/absolute/path/db.db"),
        };
        assert_eq!(config.database_path(), PathBuf::from("/absolute/path/db.db"));
    }
}
