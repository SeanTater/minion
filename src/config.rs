use crate::resources::GameConfig;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to get config directory")]
    ConfigDirNotFound,
    
    #[error("Failed to create config directory: {0}")]
    ConfigDirCreationFailed(#[from] std::io::Error),
    
    #[error("Failed to serialize config: {0}")]
    SerializationFailed(#[from] toml::ser::Error),
    
    #[error("Failed to deserialize config: {0}")]
    DeserializationFailed(#[from] toml::de::Error),
    
    #[error("Config file not found at path: {path}")]
    ConfigFileNotFound { path: PathBuf },
}

pub fn get_config_path() -> Result<PathBuf, ConfigError> {
    let mut path = dirs::config_dir().ok_or(ConfigError::ConfigDirNotFound)?;
    path.push("minion");
    fs::create_dir_all(&path)?;
    path.push("config.toml");
    Ok(path)
}

pub fn load_config() -> Result<GameConfig, ConfigError> {
    let config_path = get_config_path()?;
    
    let contents = fs::read_to_string(&config_path)
        .map_err(|_| ConfigError::ConfigFileNotFound { path: config_path })?;
    
    let config = toml::from_str::<GameConfig>(&contents)?;
    Ok(config)
}

pub fn load_config_or_default() -> GameConfig {
    load_config().unwrap_or_else(|err| {
        eprintln!("Warning: Failed to load config ({}), using defaults", err);
        GameConfig::default()
    })
}

pub fn save_config(config: &GameConfig) -> Result<(), ConfigError> {
    let config_path = get_config_path()?;
    let contents = toml::to_string_pretty(config)?;
    fs::write(config_path, contents)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::ErrorKind;

    #[test]
    fn test_config_error_display() {
        let err = ConfigError::ConfigDirNotFound;
        assert_eq!(err.to_string(), "Failed to get config directory");
        
        let io_err = std::io::Error::new(ErrorKind::PermissionDenied, "Permission denied");
        let err = ConfigError::ConfigDirCreationFailed(io_err);
        assert!(err.to_string().contains("Failed to create config directory"));
        
        let path = PathBuf::from("/fake/path");
        let err = ConfigError::ConfigFileNotFound { path: path.clone() };
        assert!(err.to_string().contains("/fake/path"));
    }

    #[test]
    fn test_load_config_or_default_fallback() {
        // Test that load_config_or_default handles errors gracefully
        // This test verifies the function exists and returns a valid GameConfig
        let config = load_config_or_default();
        
        // Should always return a valid config (either loaded or default)
        assert!(config.score >= 0); // Score should be non-negative
        
        // Username can be empty (default) or non-empty (loaded from file)
        // We don't assert specific values since this may load a real config
    }
}