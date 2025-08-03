use bevy::prelude::*;
use std::path::PathBuf;
use thiserror::Error;

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

    #[error("Invalid configuration values")]
    InvalidConfig,

    // Game-related errors
    #[error("Invalid spawn position: {position:?}")]
    InvalidSpawnPosition { position: Vec3 },

    // Map-related errors
    #[error("Invalid map data: {reason}")]
    InvalidMapData { reason: String },

    #[error("Map file not found at path: {path}")]
    MapFileNotFound { path: PathBuf },

    #[error("Corrupted map file: {reason}")]
    CorruptedMapFile { reason: String },

    #[error("Invalid terrain data: {reason}")]
    InvalidTerrainData { reason: String },

    #[error("Invalid spawn zone data: {reason}")]
    InvalidSpawnZoneData { reason: String },

    #[error("Map validation failed: {reason}")]
    MapValidationFailed { reason: String },
}

/// Result type alias for all operations
pub type MinionResult<T> = Result<T, MinionError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minion_error_display() {
        let err = MinionError::InvalidSpawnPosition {
            position: Vec3::new(100.0, 0.0, 100.0),
        };
        assert!(err.to_string().contains("Invalid spawn position"));

        let err = MinionError::ConfigDirNotFound;
        assert_eq!(err.to_string(), "Failed to get config directory");
    }
}
