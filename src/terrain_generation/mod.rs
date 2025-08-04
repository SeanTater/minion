use crate::game_logic::errors::MinionResult;
use crate::map::TerrainData;
use noise::{MultiFractal, NoiseFn, Perlin, RidgedMulti};

/// Terrain generation algorithms
#[derive(Debug, Clone)]
pub enum TerrainAlgorithm {
    Flat {
        height: f32,
    },
    Perlin {
        amplitude: f32,
        frequency: f32,
        octaves: u32,
    },
    Ridged {
        amplitude: f32,
        frequency: f32,
        octaves: u32,
    },
}

/// Main terrain generator struct
#[derive(Debug, Clone)]
pub struct TerrainGenerator {
    pub seed: u32,
    pub algorithm: TerrainAlgorithm,
}

impl TerrainGenerator {
    /// Create a new terrain generator
    pub fn new(seed: u32, algorithm: TerrainAlgorithm) -> Self {
        Self { seed, algorithm }
    }

    /// Generate terrain using the configured algorithm
    pub fn generate(&self, width: u32, height: u32, scale: f32) -> MinionResult<TerrainData> {
        let total_points = (width * height) as usize;
        let mut heights = Vec::with_capacity(total_points);

        match &self.algorithm {
            TerrainAlgorithm::Flat { height } => {
                heights.resize(total_points, *height);
            }
            TerrainAlgorithm::Perlin {
                amplitude,
                frequency,
                octaves,
            } => {
                let perlin = Perlin::new(self.seed);
                let freq_scale = scale as f64 * *frequency as f64;

                for y in 0..height {
                    let world_y = y as f64 * freq_scale;
                    for x in 0..width {
                        let world_x = x as f64 * freq_scale;

                        let mut noise_value = 0.0;
                        let mut current_amplitude = *amplitude as f64;
                        let mut current_frequency = 1.0;

                        for _ in 0..*octaves {
                            noise_value += perlin
                                .get([world_x * current_frequency, world_y * current_frequency])
                                * current_amplitude;
                            current_amplitude *= 0.5; // Persistence
                            current_frequency *= 2.0; // Lacunarity
                        }

                        heights.push(noise_value as f32);
                    }
                }
            }
            TerrainAlgorithm::Ridged {
                amplitude,
                frequency,
                octaves,
            } => {
                let ridged = RidgedMulti::<Perlin>::new(self.seed)
                    .set_octaves(*octaves as usize)
                    .set_frequency(*frequency as f64);

                for y in 0..height {
                    let world_y = y as f64 * scale as f64;
                    for x in 0..width {
                        let world_x = x as f64 * scale as f64;
                        heights.push((ridged.get([world_x, world_y]) * *amplitude as f64) as f32);
                    }
                }
            }
        }

        TerrainData::new(width, height, heights, scale)
    }
}

/// Get a predefined terrain preset
pub fn get_terrain_preset(name: &str, seed: Option<u32>) -> Option<TerrainGenerator> {
    let seed = seed.unwrap_or_else(rand::random);

    match name {
        "flat" => Some(TerrainGenerator::new(
            seed,
            TerrainAlgorithm::Flat { height: 0.0 },
        )),
        "hills" => Some(TerrainGenerator::new(
            seed,
            TerrainAlgorithm::Perlin {
                amplitude: 15.0,
                frequency: 0.01,
                octaves: 4,
            },
        )),
        "mountains" => Some(TerrainGenerator::new(
            seed,
            TerrainAlgorithm::Ridged {
                amplitude: 20.0,
                frequency: 0.005,
                octaves: 5,
            },
        )),
        "valleys" => Some(TerrainGenerator::new(
            seed,
            TerrainAlgorithm::Ridged {
                amplitude: -20.0, // Negative amplitude creates valleys
                frequency: 0.008,
                octaves: 4,
            },
        )),
        _ => None,
    }
}

/// Calculate slope at a given grid position
pub fn calculate_slope(terrain: &TerrainData, x: u32, y: u32) -> f32 {
    // Get height at center position
    let center_idx = (y * terrain.width + x) as usize;
    let h_center = terrain.heights.get(center_idx).copied().unwrap_or(0.0);

    // Get height to the right
    let right_idx = (y * terrain.width + (x + 1).min(terrain.width - 1)) as usize;
    let h_right = terrain.heights.get(right_idx).copied().unwrap_or(h_center);

    // Get height above
    let up_idx = ((y + 1).min(terrain.height - 1) * terrain.width + x) as usize;
    let h_up = terrain.heights.get(up_idx).copied().unwrap_or(h_center);

    let dx = (h_right - h_center) / terrain.scale;
    let dy = (h_up - h_center) / terrain.scale;

    (dx * dx + dy * dy).sqrt()
}

/// Check if a terrain position is suitable for spawning
pub fn is_suitable_for_spawning(
    terrain: &TerrainData,
    world_x: f32,
    world_z: f32,
    max_slope: f32,
) -> bool {
    use crate::terrain::coordinates::{get_height_at_grid, is_valid_grid, world_to_grid};

    let (grid_x, grid_z) = world_to_grid(terrain, world_x, world_z);

    // Check bounds (need margin for slope calculation)
    if !is_valid_grid(terrain, grid_x, grid_z)
        || grid_x >= (terrain.width - 1) as f32
        || grid_z >= (terrain.height - 1) as f32
    {
        return false;
    }

    let grid_x = grid_x as u32;
    let grid_z = grid_z as u32;

    // Check slope
    let slope = calculate_slope(terrain, grid_x, grid_z);
    if slope > max_slope {
        return false;
    }

    // Check height is reasonable (not too extreme)
    if let Some(height) = get_height_at_grid(terrain, grid_x, grid_z) {
        height > -50.0 && height < 100.0
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flat_terrain_generation() {
        let generator = TerrainGenerator::new(12345, TerrainAlgorithm::Flat { height: 5.0 });

        let terrain = generator
            .generate(10, 10, 1.0)
            .expect("Terrain generation should succeed with valid parameters");

        assert_eq!(terrain.width, 10);
        assert_eq!(terrain.height, 10);
        assert_eq!(terrain.heights.len(), 100);

        // All heights should be 5.0
        for &height in &terrain.heights {
            assert_eq!(height, 5.0);
        }
    }

    #[test]
    fn test_perlin_terrain_generation() {
        let generator = TerrainGenerator::new(
            12345,
            TerrainAlgorithm::Perlin {
                amplitude: 10.0,
                frequency: 0.1,
                octaves: 2,
            },
        );

        let terrain = generator
            .generate(8, 8, 1.0)
            .expect("Terrain generation should succeed with valid parameters");

        assert_eq!(terrain.width, 8);
        assert_eq!(terrain.height, 8);
        assert_eq!(terrain.heights.len(), 64);

        // Heights should vary (not all the same)
        let first_height = terrain.heights[0];
        let has_variation = terrain
            .heights
            .iter()
            .any(|&h| (h - first_height).abs() > 0.1);
        assert!(has_variation, "Perlin noise should create height variation");
    }

    #[test]
    fn test_terrain_presets() {
        let flat = get_terrain_preset("flat", Some(123)).expect("Flat terrain preset should exist");
        let hills =
            get_terrain_preset("hills", Some(123)).expect("Hills terrain preset should exist");
        let mountains = get_terrain_preset("mountains", Some(123))
            .expect("Mountains terrain preset should exist");
        let valleys =
            get_terrain_preset("valleys", Some(123)).expect("Valleys terrain preset should exist");

        assert_eq!(flat.seed, 123);
        assert_eq!(hills.seed, 123);
        assert_eq!(mountains.seed, 123);
        assert_eq!(valleys.seed, 123);

        assert!(get_terrain_preset("invalid", Some(123)).is_none());
    }

    #[test]
    fn test_slope_calculation() {
        // Create a simple terrain with known slopes
        let heights = vec![0.0, 5.0, 10.0, 0.0, 5.0, 10.0, 0.0, 5.0, 10.0];
        let terrain = TerrainData::new(3, 3, heights, 1.0)
            .expect("TerrainData creation should succeed with valid parameters");

        // Test slope calculation - should have horizontal gradient
        let slope_00 = calculate_slope(&terrain, 0, 0);
        let slope_01 = calculate_slope(&terrain, 0, 1);

        assert!(
            slope_00 > 0.0,
            "Should have positive slope due to horizontal gradient, got {}",
            slope_00
        );
        assert!(
            slope_01 >= 0.0,
            "Should have non-negative slope, got {}",
            slope_01
        );
    }

    #[test]
    fn test_spawn_suitability() {
        // Create flat terrain - should be suitable
        let heights = vec![0.0; 100];
        let terrain = TerrainData::new(10, 10, heights, 1.0).unwrap();

        assert!(is_suitable_for_spawning(&terrain, 0.0, 0.0, 0.5)); // Use center position

        // Create steep terrain - should not be suitable
        let mut steep_heights = Vec::new();
        for y in 0..10 {
            for x in 0..10 {
                steep_heights.push((x * y) as f32 * 5.0); // Creates steep gradients
            }
        }
        let steep_terrain = TerrainData::new(10, 10, steep_heights, 1.0).unwrap();

        // Some positions should be unsuitable due to steepness
        let unsuitable = !is_suitable_for_spawning(&steep_terrain, 3.0, 3.0, 0.1);
        assert!(
            unsuitable,
            "Steep terrain should be unsuitable for spawning"
        );
    }

    #[test]
    fn test_integration_with_terrain_mesh() {
        use crate::terrain::{
            generate_terrain_collider, generate_terrain_mesh, generate_terrain_mesh_and_collider,
        };

        // Create a terrain generator with Perlin noise
        let generator = TerrainGenerator::new(
            12345,
            TerrainAlgorithm::Perlin {
                amplitude: 5.0,
                frequency: 0.1,
                octaves: 3,
            },
        );

        // Generate a small terrain
        let terrain = generator.generate(16, 16, 1.0).unwrap();
        assert_eq!(terrain.width, 16);
        assert_eq!(terrain.height, 16);
        assert_eq!(terrain.heights.len(), 256);

        // Test mesh generation works with generated terrain
        let mesh = generate_terrain_mesh(&terrain).unwrap();
        assert!(mesh.count_vertices() > 0);

        // Test collider generation works with generated terrain
        let _collider = generate_terrain_collider(&terrain).unwrap();

        // Test combined generation works
        let (mesh2, _collider2) = generate_terrain_mesh_and_collider(&terrain).unwrap();
        assert!(mesh2.count_vertices() > 0);
    }
}
