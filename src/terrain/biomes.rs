use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Different biome types that can exist in the world
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BiomeType {
    Plains,
    Forest,
    Mountains,
    Desert,
    Swamp,
    Tundra,
    Ocean,
}

/// Surface material types with varying properties
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SurfaceType {
    Grass,
    Dirt,
    Sand,
    Ice,
    Water,
    Rock(RockSize),
}

/// Rock size categories for realistic distribution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RockSize {
    Pebbles,    // 0.1-0.3m
    SmallRocks, // 0.3-0.8m
    MediumRocks,// 0.8-1.5m
    LargeRocks, // 1.5-3.0m
    Boulders,   // 3.0m+
}

/// Biome configuration defining characteristics and generation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeConfig {
    pub name: String,
    pub biome_type: BiomeType,
    pub primary_surface: SurfaceType,
    pub secondary_surfaces: Vec<(SurfaceType, f32)>, // (surface, probability)
    pub elevation_preference: (f32, f32), // (min, max) normalized height
    pub slope_tolerance: f32, // Maximum slope this biome tolerates
    pub temperature: f32, // -1.0 (cold) to 1.0 (hot)
    pub humidity: f32,    // -1.0 (dry) to 1.0 (wet)
}

/// Weighted biome influence at a specific location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeBlend {
    pub weights: Vec<(BiomeType, f32)>, // Up to 4 biomes with weights summing to 1.0
}

/// Biome map storing blend information for terrain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeMap {
    pub width: u32,
    pub height: u32,
    pub blends: Vec<BiomeBlend>, // Flattened 2D array (row-major)
    pub scale: f32, // World units per grid cell (matches TerrainData)
}

/// Combined biome data including discrete map and smooth blends
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeData {
    pub biome_map: Vec<Vec<BiomeType>>, // Discrete biome assignment for pathfinding
    pub blend_map: BiomeMap, // Smooth biome blends for rendering
}

impl BiomeConfig {
    /// Create a new biome configuration
    pub fn new(
        name: String,
        biome_type: BiomeType,
        primary_surface: SurfaceType,
        elevation_preference: (f32, f32),
        slope_tolerance: f32,
        temperature: f32,
        humidity: f32,
    ) -> Self {
        Self {
            name,
            biome_type,
            primary_surface,
            secondary_surfaces: Vec::new(),
            elevation_preference,
            slope_tolerance,
            temperature,
            humidity,
        }
    }

    /// Add a secondary surface type with probability
    pub fn with_secondary_surface(mut self, surface: SurfaceType, probability: f32) -> Self {
        self.secondary_surfaces.push((surface, probability));
        self
    }

    /// Check if this biome is suitable for given elevation and slope
    pub fn is_suitable(&self, elevation: f32, slope: f32) -> f32 {
        let elevation_score = if elevation >= self.elevation_preference.0 
            && elevation <= self.elevation_preference.1 {
            1.0
        } else {
            let distance = if elevation < self.elevation_preference.0 {
                self.elevation_preference.0 - elevation
            } else {
                elevation - self.elevation_preference.1
            };
            (1.0 - distance).max(0.0)
        };

        let slope_score = if slope <= self.slope_tolerance {
            1.0
        } else {
            (1.0 - (slope - self.slope_tolerance)).max(0.0)
        };

        elevation_score * slope_score
    }
}

impl BiomeBlend {
    /// Create a new biome blend with a single biome
    pub fn single(biome_type: BiomeType) -> Self {
        Self {
            weights: vec![(biome_type, 1.0)],
        }
    }

    /// Create a blend from multiple biomes (weights will be normalized)
    pub fn from_weights(mut weights: Vec<(BiomeType, f32)>) -> Self {
        // Remove zero weights
        weights.retain(|(_, weight)| *weight > 0.0);
        
        // Normalize weights to sum to 1.0
        let total: f32 = weights.iter().map(|(_, w)| *w).sum();
        if total > 0.0 {
            for (_, weight) in &mut weights {
                *weight /= total;
            }
        }

        // Keep only top 4 weights for efficiency
        weights.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        weights.truncate(4);

        Self { weights }
    }

    /// Get the dominant biome type
    pub fn dominant_biome(&self) -> Option<BiomeType> {
        self.weights.first().map(|(biome, _)| *biome)
    }

    /// Get weight for a specific biome type
    pub fn get_weight(&self, biome_type: BiomeType) -> f32 {
        self.weights
            .iter()
            .find(|(bt, _)| *bt == biome_type)
            .map(|(_, weight)| *weight)
            .unwrap_or(0.0)
    }
}

impl BiomeMap {
    /// Create a new biome map with single biome
    pub fn uniform(width: u32, height: u32, biome_type: BiomeType, scale: f32) -> Self {
        let blend = BiomeBlend::single(biome_type);
        let blends = vec![blend; (width * height) as usize];
        
        Self {
            width,
            height,
            blends,
            scale,
        }
    }

    /// Get biome blend at grid position
    pub fn get_blend_at_grid(&self, x: u32, y: u32) -> Option<&BiomeBlend> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let index = (y * self.width + x) as usize;
        self.blends.get(index)
    }

    /// Get biome blend at world position
    pub fn get_blend_at_world(&self, world_x: f32, world_z: f32) -> Option<&BiomeBlend> {
        // Convert world coordinates to grid coordinates (centered)
        let half_width = (self.width as f32 * self.scale) / 2.0;
        let half_height = (self.height as f32 * self.scale) / 2.0;
        
        let grid_x = ((world_x + half_width) / self.scale).floor() as i32;
        let grid_z = ((world_z + half_height) / self.scale).floor() as i32;
        
        if grid_x < 0 || grid_z < 0 || grid_x >= self.width as i32 || grid_z >= self.height as i32 {
            return None;
        }
        
        self.get_blend_at_grid(grid_x as u32, grid_z as u32)
    }
}

/// Default biome configurations
pub fn create_default_biomes() -> HashMap<BiomeType, BiomeConfig> {
    let mut biomes = HashMap::new();

    biomes.insert(
        BiomeType::Plains,
        BiomeConfig::new(
            "Plains".to_string(),
            BiomeType::Plains,
            SurfaceType::Grass,
            (0.0, 0.3), // Low to moderate elevation
            0.3, // Gentle slopes
            0.1, // Mild temperature
            0.2, // Moderate humidity
        )
        .with_secondary_surface(SurfaceType::Dirt, 0.2)
        .with_secondary_surface(SurfaceType::Rock(RockSize::SmallRocks), 0.1)
    );

    biomes.insert(
        BiomeType::Forest,
        BiomeConfig::new(
            "Forest".to_string(),
            BiomeType::Forest,
            SurfaceType::Dirt,
            (0.1, 0.6), // Low to high elevation
            0.5, // Moderate slopes
            0.0, // Cool temperature
            0.5, // High humidity
        )
        .with_secondary_surface(SurfaceType::Grass, 0.3)
        .with_secondary_surface(SurfaceType::Rock(RockSize::MediumRocks), 0.15)
    );

    biomes.insert(
        BiomeType::Mountains,
        BiomeConfig::new(
            "Mountains".to_string(),
            BiomeType::Mountains,
            SurfaceType::Rock(RockSize::LargeRocks),
            (0.4, 1.0), // High elevation
            1.0, // Steep slopes OK
            -0.3, // Cold temperature
            -0.2, // Low humidity
        )
        .with_secondary_surface(SurfaceType::Rock(RockSize::Boulders), 0.3)
        .with_secondary_surface(SurfaceType::Ice, 0.2)
    );

    biomes.insert(
        BiomeType::Desert,
        BiomeConfig::new(
            "Desert".to_string(),
            BiomeType::Desert,
            SurfaceType::Sand,
            (0.0, 0.4), // Low to moderate elevation
            0.4, // Moderate slopes
            0.7, // Hot temperature
            -0.8, // Very dry
        )
        .with_secondary_surface(SurfaceType::Rock(RockSize::MediumRocks), 0.2)
        .with_secondary_surface(SurfaceType::Dirt, 0.1)
    );

    biomes.insert(
        BiomeType::Swamp,
        BiomeConfig::new(
            "Swamp".to_string(),
            BiomeType::Swamp,
            SurfaceType::Water,
            (-0.1, 0.2), // Low elevation (near water level)
            0.2, // Very gentle slopes
            0.3, // Warm temperature
            0.9, // Very humid
        )
        .with_secondary_surface(SurfaceType::Dirt, 0.4)
        .with_secondary_surface(SurfaceType::Grass, 0.2)
    );

    biomes.insert(
        BiomeType::Tundra,
        BiomeConfig::new(
            "Tundra".to_string(),
            BiomeType::Tundra,
            SurfaceType::Ice,
            (0.2, 0.8), // Moderate to high elevation
            0.3, // Gentle slopes
            -0.8, // Very cold
            -0.3, // Low humidity
        )
        .with_secondary_surface(SurfaceType::Rock(RockSize::Boulders), 0.3)
        .with_secondary_surface(SurfaceType::Dirt, 0.1)
    );

    biomes.insert(
        BiomeType::Ocean,
        BiomeConfig::new(
            "Ocean".to_string(),
            BiomeType::Ocean,
            SurfaceType::Water,
            (-1.0, 0.0), // Below sea level
            0.1, // Very flat
            0.2, // Moderate temperature
            1.0, // Maximum humidity
        )
        .with_secondary_surface(SurfaceType::Sand, 0.2)
    );

    biomes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_biome_blend_creation() {
        let blend = BiomeBlend::single(BiomeType::Forest);
        assert_eq!(blend.dominant_biome(), Some(BiomeType::Forest));
        assert_eq!(blend.get_weight(BiomeType::Forest), 1.0);
        assert_eq!(blend.get_weight(BiomeType::Plains), 0.0);
    }

    #[test]
    fn test_biome_blend_normalization() {
        let weights = vec![
            (BiomeType::Forest, 3.0),
            (BiomeType::Plains, 1.0),
        ];
        let blend = BiomeBlend::from_weights(weights);
        
        assert_eq!(blend.get_weight(BiomeType::Forest), 0.75);
        assert_eq!(blend.get_weight(BiomeType::Plains), 0.25);
    }

    #[test]
    fn test_biome_map_uniform() {
        let map = BiomeMap::uniform(10, 10, BiomeType::Plains, 1.0);
        assert_eq!(map.width, 10);
        assert_eq!(map.height, 10);
        assert_eq!(map.blends.len(), 100);
        
        let blend = map.get_blend_at_grid(5, 5).unwrap();
        assert_eq!(blend.dominant_biome(), Some(BiomeType::Plains));
    }

    #[test]
    fn test_biome_config_suitability() {
        let config = BiomeConfig::new(
            "Test".to_string(),
            BiomeType::Plains,
            SurfaceType::Grass,
            (0.2, 0.8),
            0.3,
            0.0,
            0.0,
        );

        // Perfect conditions
        assert_eq!(config.is_suitable(0.5, 0.1), 1.0);
        
        // Outside elevation range
        assert!(config.is_suitable(1.0, 0.1) < 1.0);
        
        // Too steep
        assert!(config.is_suitable(0.5, 0.8) < 1.0);
    }

    #[test]
    fn test_default_biomes() {
        let biomes = create_default_biomes();
        assert_eq!(biomes.len(), 7);
        assert!(biomes.contains_key(&BiomeType::Plains));
        assert!(biomes.contains_key(&BiomeType::Mountains));
    }
}