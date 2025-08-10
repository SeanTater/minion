use crate::game_logic::errors::MinionResult;
use crate::map::TerrainData;
use crate::terrain::biomes::{BiomeData, BiomeType};
use crate::terrain::constants::*;
use crate::terrain::coordinates::get_height_at_grid;
use pathfinding::prelude::astar;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathPoint {
    pub x: u32,
    pub z: u32,
    pub elevation: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Path {
    pub points: Vec<PathPoint>,
    pub path_type: PathType,
    pub width: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PathType {
    MainRoad,
    Trail,
    RiverPath,
    MountainPass,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathNetwork {
    pub paths: Vec<Path>,
    pub junctions: Vec<PathPoint>,
}

#[derive(Debug, Clone)]
pub struct PathGenerationConfig {
    pub main_roads: u32,
    pub trails_per_biome: u32,
    pub min_path_length: u32,
    pub max_slope_gradient: f32,
    pub avoid_water_penalty: f32,
    pub prefer_flat_bonus: f32,
}

impl Default for PathGenerationConfig {
    fn default() -> Self {
        Self {
            main_roads: DEFAULT_MAIN_ROADS,
            trails_per_biome: DEFAULT_TRAILS_PER_BIOME,
            min_path_length: DEFAULT_MIN_PATH_LENGTH,
            max_slope_gradient: DEFAULT_MAX_SLOPE_GRADIENT,
            avoid_water_penalty: 100.0,
            prefer_flat_bonus: FALLBACK_BIOME_SUITABILITY,
        }
    }
}

pub struct PathGenerator {
    config: PathGenerationConfig,
    rng: Pcg64,
}

impl PathGenerator {
    pub fn new(config: PathGenerationConfig, seed: u64) -> Self {
        Self {
            config,
            rng: Pcg64::seed_from_u64(seed),
        }
    }

    pub fn generate_path_network(
        &mut self,
        terrain: &TerrainData,
        biome_data: Option<&BiomeData>,
    ) -> MinionResult<PathNetwork> {
        let mut paths = Vec::new();
        let mut junctions = Vec::new();

        if let Some(biomes) = biome_data {
            // Generate main roads connecting major biome centers
            let main_roads = self.generate_main_roads(terrain, biomes)?;
            paths.extend(main_roads);

            // Generate trails within biomes
            let trails = self.generate_biome_trails(terrain, biomes)?;
            paths.extend(trails);

            // Find and create junctions
            junctions = self.find_path_junctions(&paths);
        } else {
            // Fallback: generate simple grid-based paths
            let simple_paths = self.generate_simple_paths(terrain)?;
            paths.extend(simple_paths);
        }

        Ok(PathNetwork { paths, junctions })
    }

    fn generate_main_roads(
        &mut self,
        terrain: &TerrainData,
        biomes: &BiomeData,
    ) -> MinionResult<Vec<Path>> {
        let mut roads = Vec::new();
        let biome_centers = self.find_biome_centers(biomes);

        // Connect major biome centers with main roads
        for _ in 0..self.config.main_roads {
            if biome_centers.len() >= 2 {
                let start_idx = self.rng.gen_range(0..biome_centers.len());
                let mut end_idx = self.rng.gen_range(0..biome_centers.len());
                while end_idx == start_idx && biome_centers.len() > 1 {
                    end_idx = self.rng.gen_range(0..biome_centers.len());
                }

                let start = biome_centers[start_idx];
                let end = biome_centers[end_idx];

                if let Some(path_points) =
                    self.find_path_astar(terrain, biomes, start, end, PathType::MainRoad)
                {
                    roads.push(Path {
                        points: path_points,
                        path_type: PathType::MainRoad,
                        width: DEFAULT_PATH_WIDTH_MAIN_ROAD,
                    });
                }
            }
        }

        Ok(roads)
    }

    fn generate_biome_trails(
        &mut self,
        terrain: &TerrainData,
        biomes: &BiomeData,
    ) -> MinionResult<Vec<Path>> {
        let mut trails = Vec::new();
        let biome_regions = self.get_biome_regions(biomes);

        for (biome_type, points) in biome_regions {
            if biome_type == BiomeType::Ocean {
                continue; // Skip water biomes
            }

            for _ in 0..self.config.trails_per_biome {
                if points.len() >= 2 {
                    let start_idx = self.rng.gen_range(0..points.len());
                    let mut end_idx = self.rng.gen_range(0..points.len());
                    while end_idx == start_idx && points.len() > 1 {
                        end_idx = self.rng.gen_range(0..points.len());
                    }

                    let start = points[start_idx];
                    let end = points[end_idx];

                    let path_type = match biome_type {
                        BiomeType::Mountains => PathType::MountainPass,
                        BiomeType::Swamp => PathType::RiverPath,
                        _ => PathType::Trail,
                    };

                    if let Some(path_points) =
                        self.find_path_astar(terrain, biomes, start, end, path_type)
                    {
                        let width = match path_type {
                            PathType::MountainPass => DEFAULT_PATH_WIDTH_MOUNTAIN_PASS,
                            PathType::RiverPath => DEFAULT_PATH_WIDTH_RIVER_PATH,
                            _ => DEFAULT_PATH_WIDTH_TRAIL,
                        };

                        trails.push(Path {
                            points: path_points,
                            path_type,
                            width,
                        });
                    }
                }
            }
        }

        Ok(trails)
    }

    fn generate_simple_paths(&mut self, terrain: &TerrainData) -> MinionResult<Vec<Path>> {
        let mut paths = Vec::new();
        let width = terrain.width;
        let height = terrain.height;

        // Generate a few random paths across the terrain
        for _ in 0..3 {
            let start = (
                self.rng.gen_range(0..width / 4),
                self.rng.gen_range(0..height / 4),
            );
            let end = (
                self.rng.gen_range(3 * width / 4..width),
                self.rng.gen_range(3 * height / 4..height),
            );

            if let Some(path_points) = self.find_simple_path(terrain, start, end) {
                paths.push(Path {
                    points: path_points,
                    path_type: PathType::Trail,
                    width: 2.0,
                });
            }
        }

        Ok(paths)
    }

    fn find_path_astar(
        &self,
        terrain: &TerrainData,
        _biomes: &BiomeData,
        start: (u32, u32),
        end: (u32, u32),
        _path_type: PathType,
    ) -> Option<Vec<PathPoint>> {
        let result = astar(
            &start,
            |&(x, z)| self.get_neighbors(x, z, terrain.width, terrain.height),
            |&pos| self.heuristic(pos, end),
            |&pos| self.is_goal(pos, end),
        );

        result.map(|(path, _cost)| {
            path.into_iter()
                .map(|(x, z)| PathPoint {
                    x,
                    z,
                    elevation: get_height_at_grid(terrain, x, z).unwrap_or(FALLBACK_TERRAIN_HEIGHT),
                })
                .collect()
        })
    }

    fn find_simple_path(
        &self,
        terrain: &TerrainData,
        start: (u32, u32),
        end: (u32, u32),
    ) -> Option<Vec<PathPoint>> {
        let result = astar(
            &start,
            |&(x, z)| self.get_neighbors(x, z, terrain.width, terrain.height),
            |&pos| self.simple_heuristic(pos, end),
            |&pos| self.is_goal(pos, end),
        );

        result.map(|(path, _cost)| {
            path.into_iter()
                .map(|(x, z)| PathPoint {
                    x,
                    z,
                    elevation: get_height_at_grid(terrain, x, z).unwrap_or(FALLBACK_TERRAIN_HEIGHT),
                })
                .collect()
        })
    }

    fn get_neighbors(&self, x: u32, z: u32, width: u32, height: u32) -> Vec<((u32, u32), u32)> {
        let mut neighbors = Vec::new();
        let directions = [
            (-1, 0),
            (1, 0),
            (0, -1),
            (0, 1),
            (-1, -1),
            (1, 1),
            (-1, 1),
            (1, -1),
        ];

        for (dx, dz) in directions {
            let new_x = x as i32 + dx;
            let new_z = z as i32 + dz;

            if new_x >= 0 && new_x < width as i32 && new_z >= 0 && new_z < height as i32 {
                let cost = if dx.abs() + dz.abs() == 2 {
                    ASTAR_DIAGONAL_COST
                } else {
                    ASTAR_CARDINAL_COST
                }; // Diagonal vs cardinal
                neighbors.push(((new_x as u32, new_z as u32), cost));
            }
        }

        neighbors
    }

    fn heuristic(&self, pos: (u32, u32), goal: (u32, u32)) -> u32 {
        let dx = (pos.0 as i32 - goal.0 as i32).unsigned_abs();
        let dz = (pos.1 as i32 - goal.1 as i32).unsigned_abs();
        dx + dz // Manhattan distance
    }

    fn simple_heuristic(&self, pos: (u32, u32), goal: (u32, u32)) -> u32 {
        let dx = (pos.0 as i32 - goal.0 as i32).unsigned_abs();
        let dz = (pos.1 as i32 - goal.1 as i32).unsigned_abs();
        ((dx * dx + dz * dz) as f32).sqrt() as u32 // Euclidean distance
    }

    fn is_goal(&self, pos: (u32, u32), goal: (u32, u32)) -> bool {
        pos == goal
    }

    fn find_biome_centers(&self, biomes: &BiomeData) -> Vec<(u32, u32)> {
        let mut centers = Vec::new();
        let mut biome_points: HashMap<BiomeType, Vec<(u32, u32)>> = HashMap::new();

        // Collect all points for each biome type
        for (x, row) in biomes.biome_map.iter().enumerate() {
            for (z, &biome_type) in row.iter().enumerate() {
                biome_points
                    .entry(biome_type)
                    .or_default()
                    .push((x as u32, z as u32));
            }
        }

        // Calculate centroid for each biome
        for (biome_type, points) in biome_points {
            if biome_type != BiomeType::Ocean && !points.is_empty() {
                let sum_x: u32 = points.iter().map(|(x, _)| *x).sum();
                let sum_z: u32 = points.iter().map(|(_, z)| *z).sum();
                let center_x = sum_x / points.len() as u32;
                let center_z = sum_z / points.len() as u32;
                centers.push((center_x, center_z));
            }
        }

        centers
    }

    fn get_biome_regions(&self, biomes: &BiomeData) -> HashMap<BiomeType, Vec<(u32, u32)>> {
        let mut regions: HashMap<BiomeType, Vec<(u32, u32)>> = HashMap::new();

        for (x, row) in biomes.biome_map.iter().enumerate() {
            for (z, &biome_type) in row.iter().enumerate() {
                regions
                    .entry(biome_type)
                    .or_default()
                    .push((x as u32, z as u32));
            }
        }

        regions
    }

    fn find_path_junctions(&self, paths: &[Path]) -> Vec<PathPoint> {
        let mut junctions = Vec::new();
        let mut point_counts: HashMap<(u32, u32), u32> = HashMap::new();

        // Count how many paths pass through each point
        for path in paths {
            for point in &path.points {
                *point_counts.entry((point.x, point.z)).or_insert(0) += 1;
            }
        }

        // Points with 3+ paths are junctions
        for ((x, z), count) in point_counts {
            if count >= 3 {
                // Find the elevation from any path that passes through this point
                let elevation = paths
                    .iter()
                    .flat_map(|p| &p.points)
                    .find(|p| p.x == x && p.z == z)
                    .map(|p| p.elevation)
                    .unwrap_or(FALLBACK_TERRAIN_HEIGHT);

                junctions.push(PathPoint { x, z, elevation });
            }
        }

        junctions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain::biomes::{BiomeBlend, BiomeMap};

    // Helper functions for creating test data
    fn create_test_terrain(width: u32, height: u32) -> TerrainData {
        let size = (width * height) as usize;
        TerrainData {
            width,
            height,
            heights: vec![0.0; size],
            scale: 1.0,
        }
    }

    fn create_test_biome_data(width: u32, height: u32) -> BiomeData {
        let biome_map = vec![vec![BiomeType::Plains; height as usize]; width as usize];
        let blend_map = BiomeMap {
            width,
            height,
            blends: vec![BiomeBlend::single(BiomeType::Plains); (width * height) as usize],
            scale: 1.0,
        };
        BiomeData {
            biome_map,
            blend_map,
        }
    }

    fn create_mixed_biome_data(width: u32, height: u32) -> BiomeData {
        let mut biome_map = vec![vec![BiomeType::Plains; height as usize]; width as usize];

        // Create different biome regions
        for x in 0..width as usize {
            for z in 0..height as usize {
                biome_map[x][z] = match (x * 2 / width as usize, z * 2 / height as usize) {
                    (0, 0) => BiomeType::Plains,
                    (0, 1) => BiomeType::Forest,
                    (1, 0) => BiomeType::Mountains,
                    (1, 1) => BiomeType::Desert,
                    _ => BiomeType::Plains,
                };
            }
        }

        let blend_map = BiomeMap {
            width,
            height,
            blends: vec![BiomeBlend::single(BiomeType::Plains); (width * height) as usize],
            scale: 1.0,
        };

        BiomeData {
            biome_map,
            blend_map,
        }
    }

    // Property Tests (manual property verification)

    #[test]
    fn property_path_connectivity() {
        let mut generator = PathGenerator::new(PathGenerationConfig::default(), 42);
        let terrain = create_test_terrain(50, 50);
        let biomes = create_mixed_biome_data(50, 50);

        let network = generator
            .generate_path_network(&terrain, Some(&biomes))
            .unwrap();

        // Property: All paths should have at least 2 points (start and end)
        for path in &network.paths {
            assert!(path.points.len() >= 2, "Path should have at least 2 points");
        }

        // Property: Path points should form a connected sequence
        for path in &network.paths {
            for window in path.points.windows(2) {
                let p1 = &window[0];
                let p2 = &window[1];
                let distance =
                    ((p1.x as i32 - p2.x as i32).abs() + (p1.z as i32 - p2.z as i32).abs()) as u32;
                assert!(
                    distance <= 2,
                    "Adjacent path points should be neighbors (distance <= 2)"
                );
            }
        }
    }

    #[test]
    fn property_boundary_constraints() {
        let mut generator = PathGenerator::new(PathGenerationConfig::default(), 42);
        let terrain = create_test_terrain(20, 20);
        let biomes = create_test_biome_data(20, 20);

        let network = generator
            .generate_path_network(&terrain, Some(&biomes))
            .unwrap();

        // Property: All path points should be within terrain bounds
        for path in &network.paths {
            for point in &path.points {
                assert!(
                    point.x < terrain.width,
                    "Path point x should be within terrain width"
                );
                assert!(
                    point.z < terrain.height,
                    "Path point z should be within terrain height"
                );
            }
        }

        // Property: All junction points should be within terrain bounds
        for junction in &network.junctions {
            assert!(
                junction.x < terrain.width,
                "Junction x should be within terrain width"
            );
            assert!(
                junction.z < terrain.height,
                "Junction z should be within terrain height"
            );
        }
    }

    #[test]
    fn property_deterministic_generation() {
        let config = PathGenerationConfig::default();
        let terrain = create_test_terrain(30, 30);
        let biomes = create_test_biome_data(30, 30);

        // Generate with same seed twice
        let mut gen1 = PathGenerator::new(config.clone(), 123);
        let mut gen2 = PathGenerator::new(config, 123);

        let network1 = gen1.generate_path_network(&terrain, Some(&biomes)).unwrap();
        let network2 = gen2.generate_path_network(&terrain, Some(&biomes)).unwrap();

        // Property: Same seed should produce identical results
        assert_eq!(
            network1.paths.len(),
            network2.paths.len(),
            "Same seed should produce same number of paths"
        );

        for (path1, path2) in network1.paths.iter().zip(network2.paths.iter()) {
            assert_eq!(path1.path_type, path2.path_type, "Path types should match");
            assert_eq!(
                path1.points.len(),
                path2.points.len(),
                "Path lengths should match"
            );

            for (p1, p2) in path1.points.iter().zip(path2.points.iter()) {
                assert_eq!(p1.x, p2.x, "Path point x coordinates should match");
                assert_eq!(p1.z, p2.z, "Path point z coordinates should match");
            }
        }
    }

    #[test]
    fn property_path_smoothness() {
        let mut generator = PathGenerator::new(PathGenerationConfig::default(), 42);
        let terrain = create_test_terrain(40, 40);
        let biomes = create_test_biome_data(40, 40);

        let network = generator
            .generate_path_network(&terrain, Some(&biomes))
            .unwrap();

        // Property: Adjacent path points should be reasonably close
        for path in &network.paths {
            for window in path.points.windows(2) {
                let p1 = &window[0];
                let p2 = &window[1];
                let dx = (p1.x as i32 - p2.x as i32).abs();
                let dz = (p1.z as i32 - p2.z as i32).abs();

                // Should be within 8-connected neighborhood
                assert!(
                    dx <= 1 && dz <= 1,
                    "Adjacent path points should be in 8-connected neighborhood"
                );
            }
        }
    }

    // Unit Tests

    #[test]
    fn test_path_generation_config_default() {
        let config = PathGenerationConfig::default();
        assert_eq!(config.main_roads, DEFAULT_MAIN_ROADS);
        assert_eq!(config.trails_per_biome, DEFAULT_TRAILS_PER_BIOME);
        assert_eq!(config.min_path_length, DEFAULT_MIN_PATH_LENGTH);
        assert_eq!(config.max_slope_gradient, DEFAULT_MAX_SLOPE_GRADIENT);
    }

    #[test]
    fn test_empty_terrain() {
        let mut generator = PathGenerator::new(PathGenerationConfig::default(), 42);
        let terrain = create_test_terrain(1, 1);
        let biomes = create_test_biome_data(1, 1);

        let network = generator
            .generate_path_network(&terrain, Some(&biomes))
            .unwrap();

        // Should handle gracefully without crashing
        assert!(network.paths.is_empty() || network.paths.iter().all(|p| p.points.len() >= 1));
    }

    #[test]
    fn test_no_biome_data_fallback() {
        let mut generator = PathGenerator::new(PathGenerationConfig::default(), 42);
        let terrain = create_test_terrain(30, 30);

        let network = generator.generate_path_network(&terrain, None).unwrap();

        // Should generate simple paths without biome data
        // All paths should be trails in fallback mode
        for path in &network.paths {
            assert_eq!(
                path.path_type,
                PathType::Trail,
                "Fallback should generate trails"
            );
            assert_eq!(path.width, 2.0, "Fallback trails should have width 2.0");
        }
    }

    #[test]
    fn test_heuristic_functions() {
        let generator = PathGenerator::new(PathGenerationConfig::default(), 42);

        // Test Manhattan distance heuristic
        let h1 = generator.heuristic((0, 0), (3, 4));
        assert_eq!(h1, 7, "Manhattan distance should be |3-0| + |4-0| = 7");

        let h2 = generator.heuristic((5, 5), (2, 1));
        assert_eq!(h2, 7, "Manhattan distance should be |2-5| + |1-5| = 7");

        // Test Euclidean distance heuristic
        let h3 = generator.simple_heuristic((0, 0), (3, 4));
        assert_eq!(h3, 5, "Euclidean distance should be sqrt(9+16) = 5");

        let h4 = generator.simple_heuristic((0, 0), (0, 0));
        assert_eq!(h4, 0, "Distance to self should be 0");
    }

    #[test]
    fn test_neighbor_generation() {
        let generator = PathGenerator::new(PathGenerationConfig::default(), 42);

        // Test center point
        let neighbors = generator.get_neighbors(5, 5, 10, 10);
        assert_eq!(neighbors.len(), 8, "Center point should have 8 neighbors");

        // Test corner point
        let neighbors = generator.get_neighbors(0, 0, 10, 10);
        assert_eq!(neighbors.len(), 3, "Corner point should have 3 neighbors");

        // Test edge point
        let neighbors = generator.get_neighbors(0, 5, 10, 10);
        assert_eq!(neighbors.len(), 5, "Edge point should have 5 neighbors");

        // Test costs are correct
        let neighbors = generator.get_neighbors(5, 5, 10, 10);
        let cardinal_neighbors = neighbors
            .iter()
            .filter(|(_, cost)| *cost == ASTAR_CARDINAL_COST)
            .count();
        let diagonal_neighbors = neighbors
            .iter()
            .filter(|(_, cost)| *cost == ASTAR_DIAGONAL_COST)
            .count();

        assert_eq!(cardinal_neighbors, 4, "Should have 4 cardinal neighbors");
        assert_eq!(diagonal_neighbors, 4, "Should have 4 diagonal neighbors");
    }

    #[test]
    fn test_goal_detection() {
        let generator = PathGenerator::new(PathGenerationConfig::default(), 42);

        assert!(
            generator.is_goal((5, 5), (5, 5)),
            "Point should be goal of itself"
        );
        assert!(
            !generator.is_goal((5, 5), (5, 6)),
            "Different points should not be goals"
        );
    }

    #[test]
    fn test_biome_center_calculation() {
        let generator = PathGenerator::new(PathGenerationConfig::default(), 42);

        // Create biome data with known centers
        let mut biome_map = vec![vec![BiomeType::Ocean; 10]; 10];

        // Create a 3x3 plains region at (1,1) to (3,3)
        for x in 1..4 {
            for z in 1..4 {
                biome_map[x][z] = BiomeType::Plains;
            }
        }

        let blend_map = BiomeMap {
            width: 10,
            height: 10,
            blends: vec![BiomeBlend::single(BiomeType::Ocean); 100],
            scale: 1.0,
        };

        let biomes = BiomeData {
            biome_map,
            blend_map,
        };
        let centers = generator.find_biome_centers(&biomes);

        // Should find the center of the plains region
        assert!(!centers.is_empty(), "Should find biome centers");

        // The center should be around (2, 2) for the 3x3 plains region
        let plains_center = centers.iter().find(|&&(x, z)| x == 2 && z == 2);
        assert!(
            plains_center.is_some(),
            "Should find plains center at approximately (2, 2)"
        );
    }

    #[test]
    fn test_junction_detection() {
        let generator = PathGenerator::new(PathGenerationConfig::default(), 42);

        // Create paths that intersect
        let path1 = Path {
            points: vec![
                PathPoint {
                    x: 0,
                    z: 5,
                    elevation: 0.0,
                },
                PathPoint {
                    x: 5,
                    z: 5,
                    elevation: 0.0,
                },
                PathPoint {
                    x: 10,
                    z: 5,
                    elevation: 0.0,
                },
            ],
            path_type: PathType::MainRoad,
            width: 4.0,
        };

        let path2 = Path {
            points: vec![
                PathPoint {
                    x: 5,
                    z: 0,
                    elevation: 0.0,
                },
                PathPoint {
                    x: 5,
                    z: 5,
                    elevation: 0.0,
                },
                PathPoint {
                    x: 5,
                    z: 10,
                    elevation: 0.0,
                },
            ],
            path_type: PathType::Trail,
            width: 2.0,
        };

        let path3 = Path {
            points: vec![
                PathPoint {
                    x: 3,
                    z: 5,
                    elevation: 0.0,
                },
                PathPoint {
                    x: 5,
                    z: 5,
                    elevation: 0.0,
                },
                PathPoint {
                    x: 7,
                    z: 5,
                    elevation: 0.0,
                },
            ],
            path_type: PathType::Trail,
            width: 2.0,
        };

        let paths = vec![path1, path2, path3];
        let junctions = generator.find_path_junctions(&paths);

        // Should find junction at (5, 5) where all three paths meet
        assert_eq!(junctions.len(), 1, "Should find exactly one junction");
        assert_eq!(junctions[0].x, 5, "Junction should be at x=5");
        assert_eq!(junctions[0].z, 5, "Junction should be at z=5");
    }
}
