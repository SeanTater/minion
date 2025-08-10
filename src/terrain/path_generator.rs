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
        let width = terrain.width as u32;
        let height = terrain.height as u32;

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
            |&(x, z)| self.get_neighbors(x, z, terrain.width as u32, terrain.height as u32),
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
            |&(x, z)| self.get_neighbors(x, z, terrain.width as u32, terrain.height as u32),
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
        let dx = (pos.0 as i32 - goal.0 as i32).abs() as u32;
        let dz = (pos.1 as i32 - goal.1 as i32).abs() as u32;
        dx + dz // Manhattan distance
    }

    fn simple_heuristic(&self, pos: (u32, u32), goal: (u32, u32)) -> u32 {
        let dx = (pos.0 as i32 - goal.0 as i32).abs() as u32;
        let dz = (pos.1 as i32 - goal.1 as i32).abs() as u32;
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
                    .or_insert_with(Vec::new)
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
                    .or_insert_with(Vec::new)
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
