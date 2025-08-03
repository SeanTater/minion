use crate::game_logic::errors::{MinionError, MinionResult};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use validator::Validate;

/// Core map definition containing all map data
#[derive(Debug, Clone, Serialize, Deserialize, Validate, Resource)]
pub struct MapDefinition {
    pub name: String,
    pub terrain: TerrainData,
    pub player_spawn: Vec3,
    pub enemy_zones: Vec<SpawnZone>,
    pub environment_objects: Vec<EnvironmentObject>,
}

/// Terrain heightmap data for procedural terrain generation
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct TerrainData {
    #[validate(range(min = 1, max = 2048))]
    pub width: u32,
    #[validate(range(min = 1, max = 2048))]
    pub height: u32,
    pub heights: Vec<f32>, // Flattened 2D array (row-major)
    #[validate(range(min = 0.1, max = 100.0))]
    pub scale: f32, // World units per grid cell
}

/// Spawn zones for enemies with configurable parameters
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SpawnZone {
    pub center: Vec3,
    #[validate(range(min = 1.0, max = 100.0))]
    pub radius: f32,
    #[validate(range(min = 1, max = 100))]
    pub max_enemies: u32,
    pub enemy_types: Vec<String>, // ["dark-knight", etc.]
}

/// Environment objects (trees, rocks, decorations, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentObject {
    pub object_type: String,
    pub position: Vec3,
    pub rotation: Vec3, // Euler angles in radians
    pub scale: Vec3,
}

impl MapDefinition {
    /// Create a new map definition with validation
    pub fn new(
        name: String,
        terrain: TerrainData,
        player_spawn: Vec3,
        enemy_zones: Vec<SpawnZone>,
        environment_objects: Vec<EnvironmentObject>,
    ) -> MinionResult<Self> {
        let map = Self {
            name,
            terrain,
            player_spawn,
            enemy_zones,
            environment_objects,
        };

        map.validate().map_err(|_| MinionError::InvalidMapData {
            reason: "Map validation failed".to_string(),
        })?;

        Ok(map)
    }

    /// Get the maps directory path
    pub fn get_maps_dir() -> MinionResult<PathBuf> {
        std::env::current_dir()
            .map_err(MinionError::ConfigDirCreationFailed)
            .map(|dir| dir.join("maps"))
    }

    /// Load a map from the maps directory
    pub fn load_from_file<P: AsRef<Path>>(filename: P) -> MinionResult<Self> {
        let maps_dir = Self::get_maps_dir()?;
        let file_path = maps_dir.join(filename);

        if !file_path.exists() {
            return Err(MinionError::MapFileNotFound { path: file_path });
        }

        let data = std::fs::read(&file_path).map_err(MinionError::ConfigDirCreationFailed)?;

        let (map, _): (MapDefinition, usize) =
            bincode::serde::decode_from_slice(&data, bincode::config::standard()).map_err(|e| {
                MinionError::CorruptedMapFile {
                    reason: format!("Failed to deserialize map data: {e}"),
                }
            })?;

        // Validate the loaded map with detailed error reporting
        map.validate().map_err(|validation_errors| {
            let error_details = validation_errors
                .field_errors()
                .iter()
                .map(|(field, errors)| {
                    let error_msgs: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
                    format!("{field}: {}", error_msgs.join(", "))
                })
                .collect::<Vec<String>>()
                .join("; ");
            
            MinionError::MapValidationFailed {
                reason: format!("Map validation failed: {error_details}"),
            }
        })?;

        Ok(map)
    }

    /// Save the map to the maps directory
    pub fn save_to_file<P: AsRef<Path>>(&self, filename: P) -> MinionResult<()> {
        // Validate before saving
        self.validate().map_err(|_| MinionError::InvalidMapData {
            reason: "Map validation failed before save".to_string(),
        })?;

        let maps_dir = Self::get_maps_dir()?;
        let file_path = maps_dir.join(filename);

        // Create parent directories for the file path if they don't exist
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).map_err(MinionError::ConfigDirCreationFailed)?;
        }

        let data =
            bincode::serde::encode_to_vec(self, bincode::config::standard()).map_err(|e| {
                MinionError::InvalidMapData {
                    reason: format!("Failed to serialize map: {e}"),
                }
            })?;

        std::fs::write(&file_path, data).map_err(MinionError::ConfigDirCreationFailed)?;

        Ok(())
    }

    /// Get the height at a specific grid position
    pub fn get_height_at_grid(&self, x: u32, y: u32) -> Option<f32> {
        crate::terrain::coordinates::get_height_at_grid(&self.terrain, x, y)
    }

    /// Get interpolated height at world position
    pub fn get_height_at_world(&self, world_x: f32, world_z: f32) -> Option<f32> {
        crate::terrain::coordinates::get_height_at_world_interpolated(&self.terrain, world_x, world_z)
    }
}

impl TerrainData {
    /// Create a new terrain data with validation
    pub fn new(width: u32, height: u32, heights: Vec<f32>, scale: f32) -> MinionResult<Self> {
        let expected_size = (width * height) as usize;
        if heights.len() != expected_size {
            return Err(MinionError::InvalidMapData {
                reason: format!(
                    "Heights array size {} does not match terrain dimensions {}x{} (expected {})",
                    heights.len(),
                    width,
                    height,
                    expected_size
                ),
            });
        }

        let terrain = Self {
            width,
            height,
            heights,
            scale,
        };

        terrain
            .validate()
            .map_err(|_| MinionError::InvalidMapData {
                reason: "Terrain validation failed".to_string(),
            })?;

        Ok(terrain)
    }

    /// Create flat terrain for testing
    pub fn create_flat(
        width: u32,
        height: u32,
        scale: f32,
        base_height: f32,
    ) -> MinionResult<Self> {
        let heights = vec![base_height; (width * height) as usize];
        Self::new(width, height, heights, scale)
    }
}

impl SpawnZone {
    /// Create a new spawn zone with validation
    pub fn new(
        center: Vec3,
        radius: f32,
        max_enemies: u32,
        enemy_types: Vec<String>,
    ) -> MinionResult<Self> {
        let zone = Self {
            center,
            radius,
            max_enemies,
            enemy_types,
        };

        zone.validate().map_err(|_| MinionError::InvalidMapData {
            reason: "Spawn zone validation failed".to_string(),
        })?;

        Ok(zone)
    }
}

impl EnvironmentObject {
    /// Create a new environment object
    pub fn new(object_type: String, position: Vec3, rotation: Vec3, scale: Vec3) -> Self {
        Self {
            object_type,
            position,
            rotation,
            scale,
        }
    }

    /// Create an environment object with default rotation and scale
    pub fn simple(object_type: String, position: Vec3) -> Self {
        Self::new(object_type, position, Vec3::ZERO, Vec3::ONE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terrain_data_creation() {
        let terrain = TerrainData::new(2, 2, vec![0.0, 1.0, 2.0, 3.0], 1.0).unwrap();
        assert_eq!(terrain.width, 2);
        assert_eq!(terrain.height, 2);
        assert_eq!(terrain.heights.len(), 4);
    }

    #[test]
    fn test_terrain_data_invalid_size() {
        let result = TerrainData::new(2, 2, vec![0.0, 1.0, 2.0], 1.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_flat_terrain_creation() {
        let terrain = TerrainData::create_flat(3, 3, 2.0, 5.0).unwrap();
        assert_eq!(terrain.width, 3);
        assert_eq!(terrain.height, 3);
        assert_eq!(terrain.heights.len(), 9);
        assert!(terrain.heights.iter().all(|&h| h == 5.0));
    }

    #[test]
    fn test_height_at_grid() {
        let terrain = TerrainData::new(2, 2, vec![0.0, 1.0, 2.0, 3.0], 1.0).unwrap();
        let map = MapDefinition {
            name: "test".to_string(),
            terrain,
            player_spawn: Vec3::ZERO,
            enemy_zones: vec![],
            environment_objects: vec![],
        };

        assert_eq!(map.get_height_at_grid(0, 0), Some(0.0));
        assert_eq!(map.get_height_at_grid(1, 0), Some(1.0));
        assert_eq!(map.get_height_at_grid(0, 1), Some(2.0));
        assert_eq!(map.get_height_at_grid(1, 1), Some(3.0));
        assert_eq!(map.get_height_at_grid(2, 0), None);
    }

    #[test]
    fn test_height_at_world_with_centering() {
        // Create a 3x3 terrain with scale 1.0
        // Heights array is row-major: [z=0: 0,1,2][z=1: 3,4,5][z=2: 6,7,8]
        // Terrain will be centered from (-1.5, -1.5) to (1.5, 1.5) in world space
        let terrain = TerrainData::new(
            3,
            3,
            vec![
                0.0, 1.0, 2.0, // z=0 row: x=0,1,2
                3.0, 4.0, 5.0, // z=1 row: x=0,1,2
                6.0, 7.0, 8.0, // z=2 row: x=0,1,2
            ],
            1.0,
        )
        .unwrap();
        let map = MapDefinition {
            name: "test".to_string(),
            terrain,
            player_spawn: Vec3::ZERO,
            enemy_zones: vec![],
            environment_objects: vec![],
        };

        // Test center position: world (0,0) should map to grid (1.5, 1.5)
        // This is between grid points (1,1), (2,1), (1,2), (2,2) with heights 4,5,7,8
        // Bilinear interpolation at (0.5, 0.5) gives average: (4+5+7+8)/4 = 6.0
        assert_eq!(map.get_height_at_world(0.0, 0.0), Some(6.0));

        // Test corner positions
        // World (-1.5, -1.5) -> grid (0, 0) at exact grid point -> height 0.0
        assert_eq!(map.get_height_at_world(-1.5, -1.5), Some(0.0));

        // World (-0.5, -1.5) -> grid (1, 0) at exact grid point -> height 1.0
        assert_eq!(map.get_height_at_world(-0.5, -1.5), Some(1.0));

        // World (-1.5, -0.5) -> grid (0, 1) at exact grid point -> height 3.0
        assert_eq!(map.get_height_at_world(-1.5, -0.5), Some(3.0));

        // Test out of bounds
        assert_eq!(map.get_height_at_world(-2.0, 0.0), None);
        assert_eq!(map.get_height_at_world(2.0, 0.0), None);
    }

    #[test]
    fn test_spawn_zone_creation() {
        let zone = SpawnZone::new(
            Vec3::new(0.0, 0.0, 0.0),
            5.0,
            10,
            vec!["dark-knight".to_string()],
        )
        .unwrap();

        assert_eq!(zone.radius, 5.0);
        assert_eq!(zone.max_enemies, 10);
        assert_eq!(zone.enemy_types.len(), 1);
    }

    #[test]
    fn test_environment_object_creation() {
        let obj = EnvironmentObject::simple("tree".to_string(), Vec3::new(1.0, 0.0, 1.0));
        assert_eq!(obj.object_type, "tree");
        assert_eq!(obj.position, Vec3::new(1.0, 0.0, 1.0));
        assert_eq!(obj.rotation, Vec3::ZERO);
        assert_eq!(obj.scale, Vec3::ONE);
    }

    #[test]
    fn test_map_validation() {
        let terrain = TerrainData::create_flat(10, 10, 1.0, 0.0).unwrap();
        let spawn_zone = SpawnZone::new(
            Vec3::new(5.0, 0.0, 5.0),
            3.0,
            5,
            vec!["dark-knight".to_string()],
        )
        .unwrap();

        let map = MapDefinition::new(
            "test_map".to_string(),
            terrain,
            Vec3::new(5.0, 0.0, 5.0),
            vec![spawn_zone],
            vec![],
        )
        .unwrap();

        assert_eq!(map.name, "test_map");
        assert_eq!(map.enemy_zones.len(), 1);
    }
}
