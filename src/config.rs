use crate::resources::GameConfig;
use crate::game_logic::{MinionError, MinionResult};
use std::fs;
use std::path::PathBuf;

pub fn get_config_path() -> MinionResult<PathBuf> {
    let mut path = dirs::config_dir().ok_or(MinionError::ConfigDirNotFound)?;
    path.push("minion");
    fs::create_dir_all(&path)?;
    path.push("config.toml");
    Ok(path)
}

pub fn load_config() -> MinionResult<GameConfig> {
    let config_path = get_config_path()?;
    
    let contents = fs::read_to_string(&config_path)
        .map_err(|_| MinionError::ConfigFileNotFound { path: config_path })?;
    
    let config = toml::from_str::<GameConfig>(&contents)?;
    Ok(config)
}

pub fn load_config_or_default() -> GameConfig {
    load_config().unwrap_or_else(|err| {
        eprintln!("Warning: Failed to load config ({}), using defaults", err);
        GameConfig::default()
    })
}

pub fn save_config(config: &GameConfig) -> MinionResult<()> {
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
        let err = MinionError::ConfigDirNotFound;
        assert_eq!(err.to_string(), "Failed to get config directory");
        
        let io_err = std::io::Error::new(ErrorKind::PermissionDenied, "Permission denied");
        let err = MinionError::ConfigDirCreationFailed(io_err);
        assert!(err.to_string().contains("Failed to create config directory"));
        
        let path = PathBuf::from("/fake/path");
        let err = MinionError::ConfigFileNotFound { path: path.clone() };
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