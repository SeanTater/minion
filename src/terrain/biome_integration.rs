use super::biome_generator::{BiomeGenerationConfig, BiomeGenerator};
use super::biomes::{BiomeData, BiomeMap, BiomeType, create_default_biomes};
use super::path_generator::{PathGenerationConfig, PathGenerator, PathNetwork};
use crate::game_logic::errors::MinionResult;
use crate::map::TerrainData;

/// Integration utilities for adding biome generation to existing terrain
pub struct BiomeIntegration;

impl BiomeIntegration {
    /// Generate a biome map for existing terrain data
    pub fn generate_biome_map_for_terrain(
        terrain: &TerrainData,
        seed: u32,
        region_count: Option<u32>,
    ) -> MinionResult<BiomeMap> {
        let config = BiomeGenerationConfig {
            seed,
            region_count: region_count.unwrap_or(6),
            transition_radius: 30.0, // Smaller for higher density terrain
            biome_preferences: vec![
                BiomeType::Plains,
                BiomeType::Forest,
                BiomeType::Mountains,
                BiomeType::Desert,
            ],
        };

        let biome_configs = create_default_biomes();
        let mut generator = BiomeGenerator::new(config, biome_configs);

        generator.generate(terrain)
    }

    /// Generate a simple biome map for testing (single biome)
    pub fn generate_simple_biome_map(terrain: &TerrainData, biome_type: BiomeType) -> BiomeMap {
        BiomeMap::uniform(terrain.width, terrain.height, biome_type, terrain.scale)
    }

    /// Generate complete biome data for terrain
    pub fn generate_biome_data_for_terrain(
        terrain: &TerrainData,
        seed: u32,
        region_count: Option<u32>,
    ) -> MinionResult<BiomeData> {
        let config = BiomeGenerationConfig {
            seed,
            region_count: region_count.unwrap_or(6),
            transition_radius: 30.0,
            biome_preferences: vec![
                BiomeType::Plains,
                BiomeType::Forest,
                BiomeType::Mountains,
                BiomeType::Desert,
            ],
        };

        let biome_configs = create_default_biomes();
        let mut generator = BiomeGenerator::new(config, biome_configs);
        generator.generate_biome_data(terrain)
    }

    /// Generate a path network for terrain with optional biome integration
    pub fn generate_path_network(
        terrain: &TerrainData,
        biome_data: Option<&BiomeData>,
        seed: u64,
        config: Option<PathGenerationConfig>,
    ) -> MinionResult<PathNetwork> {
        let path_config = config.unwrap_or_default();
        let mut generator = PathGenerator::new(path_config, seed);
        generator.generate_path_network(terrain, biome_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::TerrainData;

    #[test]
    fn test_simple_biome_map_generation() {
        let terrain = TerrainData::create_flat(32, 32, 1.0, 0.0).unwrap();
        let biome_map = BiomeIntegration::generate_simple_biome_map(&terrain, BiomeType::Plains);

        assert_eq!(biome_map.width, 32);
        assert_eq!(biome_map.height, 32);
        assert_eq!(biome_map.scale, 1.0);

        let blend = biome_map.get_blend_at_grid(16, 16).unwrap();
        assert_eq!(blend.dominant_biome(), Some(BiomeType::Plains));
    }

    #[test]
    fn test_complex_biome_map_generation() {
        let terrain = TerrainData::create_flat(64, 64, 0.5, 5.0).unwrap();
        let biome_map =
            BiomeIntegration::generate_biome_map_for_terrain(&terrain, 12345, Some(4)).unwrap();

        assert_eq!(biome_map.width, 64);
        assert_eq!(biome_map.height, 64);
        assert_eq!(biome_map.scale, 0.5);

        // Should have some biome data
        assert!(!biome_map.blends.is_empty());

        // Test world coordinate lookup
        let blend = biome_map.get_blend_at_world(0.0, 0.0);
        assert!(blend.is_some());
    }
}
