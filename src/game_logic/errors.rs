use thiserror::Error;
use bevy::prelude::*;
use std::path::PathBuf;

#[derive(Error, Debug)]
pub enum MinionError {
    // Config-related errors
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
    
    // Game-related errors
    #[error("Invalid spawn position: {position:?}")]
    InvalidSpawnPosition { position: Vec3 },
}

/// Result type alias for all operations
pub type MinionResult<T> = Result<T, MinionError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minion_error_display() {
        let err = MinionError::InvalidSpawnPosition { 
            position: Vec3::new(100.0, 0.0, 100.0) 
        };
        assert!(err.to_string().contains("Invalid spawn position"));
        
        let err = MinionError::ConfigDirNotFound;
        assert_eq!(err.to_string(), "Failed to get config directory");
    }
}