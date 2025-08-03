use super::biomes::{BiomeBlend, BiomeConfig, BiomeMap, BiomeType, BiomeData};
use super::constants::*;
use crate::game_logic::errors::{MinionError, MinionResult};
use crate::map::TerrainData;
use bevy::prelude::*;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use std::collections::HashMap;
use voronoice::{VoronoiBuilder, Point, BoundingBox, Voronoi};

/// Configuration for biome generation
#[derive(Debug, Clone)]
pub struct BiomeGenerationConfig {
    pub seed: u32,
    pub region_count: u32,
    pub transition_radius: f32, // World units for smooth blending
    pub biome_preferences: Vec<BiomeType>, // Preferred biomes to place
}

/// Voronoi region with biome assignment
#[derive(Debug, Clone)]
struct BiomeRegion {
    center: Point,
    biome_type: BiomeType,
    weight: f32, // Influence strength
}

/// Generator for biome maps using Voronoi diagrams
pub struct BiomeGenerator {
    config: BiomeGenerationConfig,
    biome_configs: HashMap<BiomeType, BiomeConfig>,
    rng: Pcg64,
    voronoi_cache: Option<(Voronoi, Vec<BiomeRegion>)>, // Cache the expensive Voronoi computation
}

impl Default for BiomeGenerationConfig {
    fn default() -> Self {
        Self {
            seed: 12345,
            region_count: DEFAULT_BIOME_REGIONS,
            transition_radius: DEFAULT_TRANSITION_RADIUS,
            biome_preferences: vec![
                BiomeType::Plains,
                BiomeType::Forest,
                BiomeType::Mountains,
                BiomeType::Desert,
            ],
        }
    }
}

impl BiomeGenerator {
    /// Create a new biome generator
    pub fn new(config: BiomeGenerationConfig, biome_configs: HashMap<BiomeType, BiomeConfig>) -> Self {
        let rng = Pcg64::seed_from_u64(config.seed as u64);
        Self {
            config,
            biome_configs,
            rng,
            voronoi_cache: None,
        }
    }

    /// Generate a biome map for the given terrain
    pub fn generate(&mut self, terrain: &TerrainData) -> MinionResult<BiomeMap> {
        info!("Generating biome map with {} regions", self.config.region_count);

        // Calculate world bounds
        let world_width = terrain.width as f32 * terrain.scale;
        let world_height = terrain.height as f32 * terrain.scale;
        let half_width = world_width / 2.0;
        let half_height = world_height / 2.0;

        // Get or build cached Voronoi diagram
        let (voronoi, regions) = if let Some((cached_voronoi, cached_regions)) = &self.voronoi_cache {
            (cached_voronoi, cached_regions)
        } else {
            // Generate random Voronoi sites
            let sites = self.generate_voronoi_sites(world_width, world_height)?;

            // Assign biomes to sites based on terrain characteristics
            let regions = self.assign_biomes_to_sites(&sites, terrain)?;

            // Build Voronoi diagram
            let bbox = BoundingBox::new(
                Point { x: 0.0, y: 0.0 },
                world_width as f64,
                world_height as f64,
            );
            
            let voronoi = VoronoiBuilder::default()
                .set_sites(sites)
                .set_bounding_box(bbox)
                .build()
                .ok_or_else(|| MinionError::InvalidMapData {
                    reason: "Failed to build Voronoi diagram".to_string(),
                })?;

            // Cache the results
            self.voronoi_cache = Some((voronoi, regions));
            let (cached_voronoi, cached_regions) = self.voronoi_cache.as_ref().unwrap();
            (cached_voronoi, cached_regions)
        };

        // Generate biome blend for each terrain grid cell
        let mut blends = Vec::with_capacity((terrain.width * terrain.height) as usize);

        for z in 0..terrain.height {
            for x in 0..terrain.width {
                let world_x = (x as f32 * terrain.scale) - half_width;
                let world_z = (z as f32 * terrain.scale) - half_height;
                
                let blend = self.calculate_biome_blend_at_position(
                    Point { x: world_x as f64, y: world_z as f64 },
                    regions,
                    voronoi,
                )?;
                
                blends.push(blend);
            }
        }

        Ok(BiomeMap {
            width: terrain.width,
            height: terrain.height,
            blends,
            scale: terrain.scale,
        })
    }

    /// Generate complete biome data including discrete map and smooth blends
    pub fn generate_biome_data(&mut self, terrain: &TerrainData) -> MinionResult<BiomeData> {
        // Generate the smooth blend map
        let blend_map = self.generate(terrain)?;
        
        // Create discrete biome map for pathfinding
        let mut biome_map = Vec::with_capacity(terrain.height as usize);
        for z in 0..terrain.height {
            let mut row = Vec::with_capacity(terrain.width as usize);
            for x in 0..terrain.width {
                let index = (z * terrain.width + x) as usize;
                let dominant_biome = blend_map.blends[index]
                    .dominant_biome()
                    .unwrap_or(BiomeType::Plains);
                row.push(dominant_biome);
            }
            biome_map.push(row);
        }

        Ok(BiomeData {
            biome_map,
            blend_map,
        })
    }

    /// Generate random Voronoi sites across the terrain
    fn generate_voronoi_sites(&mut self, world_width: f32, world_height: f32) -> MinionResult<Vec<Point>> {
        let mut sites = Vec::new();
        let half_width = world_width / 2.0;
        let half_height = world_height / 2.0;

        // Generate somewhat evenly distributed sites with some randomness
        let sites_per_side = (self.config.region_count as f32).sqrt().ceil() as u32;
        let cell_width = world_width / sites_per_side as f32;
        let cell_height = world_height / sites_per_side as f32;

        for i in 0..sites_per_side {
            for j in 0..sites_per_side {
                if sites.len() >= self.config.region_count as usize {
                    break;
                }

                // Calculate cell center
                let base_x = (i as f32 + 0.5) * cell_width - half_width;
                let base_y = (j as f32 + 0.5) * cell_height - half_height;

                // Add random offset within cell (up to 40% of cell size)
                let offset_x = self.rng.gen_range(-cell_width * 0.4..cell_width * 0.4);
                let offset_y = self.rng.gen_range(-cell_height * 0.4..cell_height * 0.4);

                sites.push(Point {
                    x: (base_x + offset_x) as f64,
                    y: (base_y + offset_y) as f64,
                });
            }
        }

        if sites.len() < MIN_VORONOI_SITES {
            return Err(MinionError::InvalidMapData {
                reason: format!("Need at least {} Voronoi sites for diagram generation", MIN_VORONOI_SITES),
            });
        }

        info!("Generated {} Voronoi sites", sites.len());
        Ok(sites)
    }

    /// Assign biome types to Voronoi sites based on terrain characteristics
    fn assign_biomes_to_sites(&mut self, sites: &[Point], terrain: &TerrainData) -> MinionResult<Vec<BiomeRegion>> {
        let mut regions = Vec::new();

        for (i, site) in sites.iter().enumerate() {
            // Sample terrain at site location
            let elevation = self.sample_terrain_elevation(site, terrain);
            let slope = self.sample_terrain_slope(site, terrain);

            // Normalize elevation (assuming terrain heights are roughly 0-50)
            let normalized_elevation = (elevation / 50.0).clamp(-1.0, 1.0);

            // Find best biome for this location
            let biome_type = self.select_biome_for_location(normalized_elevation, slope, i);

            regions.push(BiomeRegion {
                center: site.clone(),
                biome_type,
                weight: 1.0, // Could be adjusted based on biome strength
            });
        }

        info!("Assigned biomes to {} regions", regions.len());
        Ok(regions)
    }

    /// Sample terrain elevation at a world position
    fn sample_terrain_elevation(&self, point: &Point, terrain: &TerrainData) -> f32 {
        // Convert world coordinates to grid coordinates
        let half_width = (terrain.width as f32 * terrain.scale) / 2.0;
        let half_height = (terrain.height as f32 * terrain.scale) / 2.0;
        
        let grid_x = ((point.x as f32 + half_width) / terrain.scale).round() as i32;
        let grid_z = ((point.y as f32 + half_height) / terrain.scale).round() as i32;
        
        if grid_x < 0 || grid_z < 0 || grid_x >= terrain.width as i32 || grid_z >= terrain.height as i32 {
            return 0.0;
        }
        
        let index = (grid_z as u32 * terrain.width + grid_x as u32) as usize;
        terrain.heights.get(index).copied().unwrap_or(0.0)
    }

    /// Sample terrain slope at a world position
    fn sample_terrain_slope(&self, point: &Point, terrain: &TerrainData) -> f32 {
        // Convert world coordinates to grid coordinates
        let half_width = (terrain.width as f32 * terrain.scale) / 2.0;
        let half_height = (terrain.height as f32 * terrain.scale) / 2.0;
        
        let grid_x = ((point.x as f32 + half_width) / terrain.scale).round() as i32;
        let grid_z = ((point.y as f32 + half_height) / terrain.scale).round() as i32;
        
        if grid_x < 1 || grid_z < 1 || grid_x >= (terrain.width - 1) as i32 || grid_z >= (terrain.height - 1) as i32 {
            return 0.0;
        }
        
        crate::terrain_generation::calculate_slope(terrain, grid_x as u32, grid_z as u32)
    }

    /// Select the best biome type for a location based on characteristics
    fn select_biome_for_location(&mut self, elevation: f32, slope: f32, site_index: usize) -> BiomeType {
        // Start with preferred biomes or cycle through them
        let preferred_biome = if !self.config.biome_preferences.is_empty() {
            self.config.biome_preferences[site_index % self.config.biome_preferences.len()]
        } else {
            BiomeType::Plains
        };

        // Check if preferred biome is suitable
        if let Some(config) = self.biome_configs.get(&preferred_biome) {
            if config.is_suitable(elevation, slope) > 0.5 {
                return preferred_biome;
            }
        }

        // Find the most suitable biome
        let mut best_biome = BiomeType::Plains;
        let mut best_score = 0.0;

        for (biome_type, config) in &self.biome_configs {
            let score = config.is_suitable(elevation, slope);
            if score > best_score {
                best_score = score;
                best_biome = *biome_type;
            }
        }

        best_biome
    }

    /// Calculate biome blend at a specific world position
    fn calculate_biome_blend_at_position(
        &self,
        point: Point,
        regions: &[BiomeRegion],
        _voronoi: &voronoice::Voronoi,
    ) -> MinionResult<BiomeBlend> {
        // Calculate distances to all region centers
        let mut influences = Vec::new();

        for region in regions {
            let distance = ((point.x - region.center.x).powi(2) + (point.y - region.center.y).powi(2)).sqrt();
            
            // Apply smooth falloff based on transition radius
            let influence = if distance <= self.config.transition_radius as f64 {
                // Use smoothstep for natural falloff
                let t = distance / self.config.transition_radius as f64;
                let smooth_t = t * t * (3.0 - 2.0 * t); // smoothstep function
                ((1.0 - smooth_t) * region.weight as f64) as f32
            } else {
                // Exponential falloff beyond transition radius
                let excess = distance - self.config.transition_radius as f64;
                let falloff = (-excess / self.config.transition_radius as f64).exp();
                (falloff * region.weight as f64 * 0.1) as f32 // Reduced influence
            };

            if influence > 0.001 {
                influences.push((region.biome_type, influence));
            }
        }

        // If no influences (shouldn't happen), default to Plains
        if influences.is_empty() {
            return Ok(BiomeBlend::single(BiomeType::Plains));
        }

        // Create blend from influences
        Ok(BiomeBlend::from_weights(influences))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain::biomes::create_default_biomes;

    #[test]
    fn test_biome_generation_config() {
        let config = BiomeGenerationConfig::default();
        assert_eq!(config.region_count, DEFAULT_BIOME_REGIONS);
        assert!(config.transition_radius > 0.0);
        assert!(!config.biome_preferences.is_empty());
    }

    #[test]
    fn test_voronoi_sites_generation() {
        let config = BiomeGenerationConfig {
            region_count: 4,
            ..Default::default()
        };
        let biome_configs = create_default_biomes();
        let mut generator = BiomeGenerator::new(config, biome_configs);

        let sites = generator.generate_voronoi_sites(100.0, 100.0).unwrap();
        assert_eq!(sites.len(), 4);

        // Check that sites are within bounds
        for site in &sites {
            assert!(site.x >= -50.0 && site.x <= 50.0);
            assert!(site.y >= -50.0 && site.y <= 50.0);
        }
    }

    #[test]
    fn test_terrain_sampling() {
        use crate::map::TerrainData;

        let terrain = TerrainData::create_flat(10, 10, 1.0, 5.0).unwrap();
        let config = BiomeGenerationConfig::default();
        let biome_configs = create_default_biomes();
        let generator = BiomeGenerator::new(config, biome_configs);

        let point = Point { x: 0.0, y: 0.0 };
        let elevation = generator.sample_terrain_elevation(&point, &terrain);
        assert_eq!(elevation, 5.0);
    }
}