use bevy::prelude::*;
use minion::game_logic::errors::MinionResult;
use minion::map::{EnvironmentObject, MapDefinition, SpawnZone, TerrainData};
use minion::terrain::biome_integration::BiomeIntegration;
use minion::terrain::biomes::BiomeType;
use minion::terrain::path_generator::PathGenerationConfig;
use minion::terrain_generation::{TerrainGenerator, is_suitable_for_spawning};
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;

pub struct MapGenerationConfig {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub player_spawn: Vec3,
    pub generator: TerrainGenerator,
    pub object_density: f32,
    pub object_types: Vec<String>,
    pub scale_range: (f32, f32),
    pub terrain_scale: f32,
    pub enable_biomes: bool,
    pub biome_regions: u32,
    pub enable_paths: bool,
    pub main_roads: u32,
    pub trails_per_biome: u32,
}

pub struct MapGenerator;

impl MapGenerator {
    /// Correct spawn height to match terrain elevation at that position
    fn correct_spawn_height(terrain: &TerrainData, spawn_pos: Vec3) -> MinionResult<Vec3> {
        use minion::terrain::coordinates::get_height_at_world_interpolated;

        let terrain_height =
            get_height_at_world_interpolated(terrain, spawn_pos.x, spawn_pos.z).unwrap_or(0.0);

        // Spawn 1.0 unit above the terrain
        Ok(Vec3::new(spawn_pos.x, terrain_height + 1.0, spawn_pos.z))
    }

    pub fn generate(config: MapGenerationConfig) -> MinionResult<MapDefinition> {
        println!("Generating map: {name}", name = config.name);
        println!(
            "Terrain size: {width}x{height} grid cells",
            width = config.width,
            height = config.height
        );
        println!("Player spawn: {spawn}", spawn = config.player_spawn);
        println!(
            "Using terrain type (seed: {seed})",
            seed = config.generator.seed
        );

        // Generate terrain
        let terrain =
            config
                .generator
                .generate(config.width, config.height, config.terrain_scale)?;
        println!(
            "Generated terrain with {} height points",
            terrain.heights.len()
        );

        // Fix player spawn height to match terrain elevation
        let corrected_player_spawn = Self::correct_spawn_height(&terrain, config.player_spawn)?;
        println!(
            "Corrected player spawn from {} to {}",
            config.player_spawn, corrected_player_spawn
        );

        // Generate biome data if enabled
        let biome_data = if config.enable_biomes {
            println!("Generating biome map with {} regions", config.biome_regions);
            Some(BiomeIntegration::generate_biome_data_for_terrain(
                &terrain,
                config.generator.seed.wrapping_add(1337),
                Some(config.biome_regions),
            )?)
        } else {
            None
        };

        // Generate path network if enabled
        let _path_network = if config.enable_paths {
            println!(
                "Generating path network with {} main roads, {} trails per biome",
                config.main_roads, config.trails_per_biome
            );

            let path_config = PathGenerationConfig {
                main_roads: config.main_roads,
                trails_per_biome: config.trails_per_biome,
                ..Default::default()
            };

            Some(BiomeIntegration::generate_path_network(
                &terrain,
                biome_data.as_ref(),
                config.generator.seed.wrapping_add(2022) as u64,
                Some(path_config),
            )?)
        } else {
            None
        };

        // Generate spawn zones (biome-aware if biomes enabled)
        let enemy_zones = if let Some(ref biome_data) = biome_data {
            Self::generate_biome_aware_spawn_zones(
                &terrain,
                &biome_data.blend_map,
                corrected_player_spawn,
                5,
            )?
        } else {
            Self::generate_spawn_zones(&terrain, corrected_player_spawn, 5)?
        };
        println!(
            "Generated {count} enemy spawn zones",
            count = enemy_zones.len()
        );

        // Generate environment objects (biome-aware if biomes enabled)
        let object_seed = config.generator.seed.wrapping_add(42);
        let environment_objects = if let Some(ref biome_data) = biome_data {
            Self::generate_biome_aware_objects(
                &terrain,
                &biome_data.blend_map,
                corrected_player_spawn,
                &enemy_zones,
                config.object_density,
                &config.object_types,
                config.scale_range,
                object_seed,
            )?
        } else {
            Self::generate_objects(
                &terrain,
                corrected_player_spawn,
                &enemy_zones,
                config.object_density,
                &config.object_types,
                config.scale_range,
                object_seed,
            )?
        };

        Ok(MapDefinition::new(
            config.name,
            terrain,
            corrected_player_spawn,
            enemy_zones,
            environment_objects,
        )?)
    }

    fn generate_spawn_zones(
        terrain: &TerrainData,
        player_spawn: Vec3,
        num_zones: u32,
    ) -> MinionResult<Vec<SpawnZone>> {
        let mut zones = Vec::new();
        let (base_radius, base_max_enemies, max_slope) = (3.0, 2, 0.3);
        let mut attempts = 0;
        const MAX_ATTEMPTS: u32 = 1000;

        println!("Analyzing terrain for suitable spawn locations...");

        while zones.len() < num_zones as usize && attempts < MAX_ATTEMPTS {
            let i = zones.len() as u32;
            let position = Self::calculate_ring_position(player_spawn, i);

            if is_suitable_for_spawning(terrain, position.x, position.z, max_slope) {
                let terrain_height = minion::terrain::coordinates::get_height_at_world_nearest(
                    terrain, position.x, position.z,
                )
                .unwrap_or(0.0);

                let final_center = Vec3::new(position.x, terrain_height + 1.0, position.z);
                let radius = base_radius + (i % 3) as f32 * 0.5;
                let max_enemies = base_max_enemies + (i % 3);

                zones.push(SpawnZone::new(
                    final_center,
                    radius,
                    max_enemies,
                    vec!["dark-knight".to_string()],
                )?);

                println!(
                    "Found suitable spawn location at ({:.1}, {:.1}, {:.1})",
                    final_center.x, final_center.y, final_center.z
                );
            }
            attempts += 1;
        }

        if zones.len() < num_zones as usize {
            println!(
                "Warning: Only found {} suitable spawn locations out of {} requested",
                zones.len(),
                num_zones
            );
        }

        Ok(zones)
    }

    fn calculate_ring_position(player_spawn: Vec3, index: u32) -> Vec3 {
        let angle = (index as f32 * 2.3) % (2.0 * std::f32::consts::PI);
        let distance = 8.0 + (index % 4) as f32 * 2.0; // Reduced from 10.0 to ensure positions are within terrain
        player_spawn + Vec3::new(angle.cos() * distance, 0.0, angle.sin() * distance)
    }

    fn generate_objects(
        terrain: &TerrainData,
        player_spawn: Vec3,
        enemy_zones: &[SpawnZone],
        density: f32,
        object_types: &[String],
        scale_range: (f32, f32),
        seed: u32,
    ) -> MinionResult<Vec<EnvironmentObject>> {
        if density <= 0.0 {
            return Ok(Vec::new());
        }

        let mut rng = Pcg64::seed_from_u64(seed as u64);
        let mut objects = Vec::new();

        let terrain_area =
            (terrain.width as f32 * terrain.scale) * (terrain.height as f32 * terrain.scale);
        let max_objects = 200.min((terrain_area * density * 0.01) as u32);

        let constraints = ObjectPlacementConstraints {
            min_distance_from_spawn: 2.0,
            min_distance_from_enemies: 2.0,
            min_distance_between_objects: 1.5,
            max_slope: 0.4,
        };

        println!("Placing up to {max_objects} environment objects (density: {density})...");

        let mut attempts = 0;
        let max_attempts = max_objects * 10;

        while objects.len() < max_objects as usize && attempts < max_attempts {
            if let Some(obj) = Self::try_place_object(
                terrain,
                player_spawn,
                enemy_zones,
                &objects,
                object_types,
                scale_range,
                &constraints,
                &mut rng,
            )? {
                objects.push(obj);
            }
            attempts += 1;
        }

        println!(
            "Placed {} environment objects (attempted {} placements)",
            objects.len(),
            attempts
        );

        Ok(objects)
    }

    fn try_place_object(
        terrain: &TerrainData,
        player_spawn: Vec3,
        enemy_zones: &[SpawnZone],
        existing_objects: &[EnvironmentObject],
        object_types: &[String],
        scale_range: (f32, f32),
        constraints: &ObjectPlacementConstraints,
        rng: &mut Pcg64,
    ) -> MinionResult<Option<EnvironmentObject>> {
        let world_x = rng.gen_range(1.0..(terrain.width as f32 - 1.0) * terrain.scale);
        let world_z = rng.gen_range(1.0..(terrain.height as f32 - 1.0) * terrain.scale);

        let terrain_height = match minion::terrain::coordinates::get_height_at_world_nearest(
            terrain, world_x, world_z,
        ) {
            Some(height) => height,
            None => return Ok(None),
        };
        let candidate_pos = Vec3::new(world_x, terrain_height, world_z);

        if !Self::is_valid_placement(
            &candidate_pos,
            player_spawn,
            enemy_zones,
            existing_objects,
            terrain,
            constraints,
        ) {
            return Ok(None);
        }

        let object_type = object_types[rng.gen_range(0..object_types.len())].clone();
        let rotation = Vec3::new(0.0, rng.gen_range(0.0..2.0 * std::f32::consts::PI), 0.0);
        let scale_factor = rng.gen_range(scale_range.0..=scale_range.1);
        let scale = Vec3::splat(scale_factor);

        Ok(Some(EnvironmentObject::new(
            object_type,
            candidate_pos,
            rotation,
            scale,
        )))
    }

    fn is_valid_placement(
        candidate_pos: &Vec3,
        player_spawn: Vec3,
        enemy_zones: &[SpawnZone],
        existing_objects: &[EnvironmentObject],
        terrain: &TerrainData,
        constraints: &ObjectPlacementConstraints,
    ) -> bool {
        // Check distance constraints
        candidate_pos.distance(player_spawn) >= constraints.min_distance_from_spawn
            && enemy_zones.iter().all(|zone| {
                candidate_pos.distance(zone.center)
                    >= zone.radius + constraints.min_distance_from_enemies
            })
            && existing_objects.iter().all(|obj| {
                candidate_pos.distance(obj.position) >= constraints.min_distance_between_objects
            })
            && is_suitable_for_spawning(
                terrain,
                candidate_pos.x,
                candidate_pos.z,
                constraints.max_slope,
            )
    }
}

struct ObjectPlacementConstraints {
    min_distance_from_spawn: f32,
    min_distance_from_enemies: f32,
    min_distance_between_objects: f32,
    max_slope: f32,
}

impl MapGenerator {
    /// Generate biome-aware spawn zones
    fn generate_biome_aware_spawn_zones(
        terrain: &TerrainData,
        biome_map: &minion::terrain::biomes::BiomeMap,
        player_spawn: Vec3,
        max_zones: usize,
    ) -> MinionResult<Vec<SpawnZone>> {
        use minion::terrain::biomes::BiomeType;

        let mut zones = Vec::new();
        let mut attempts = 0;
        let max_attempts = max_zones * 10;

        while zones.len() < max_zones && attempts < max_attempts {
            attempts += 1;

            // Calculate position using ring-based placement
            let position = Self::calculate_ring_position(player_spawn, attempts as u32);

            // Check biome suitability
            if let Some(blend) = biome_map.get_blend_at_world(position.x, position.z) {
                if let Some(dominant_biome) = blend.dominant_biome() {
                    // Avoid placing spawns in water or swamp biomes
                    if matches!(dominant_biome, BiomeType::Ocean | BiomeType::Swamp) {
                        continue;
                    }
                }
            }

            // Use existing spawn zone logic for terrain checks
            if is_suitable_for_spawning(terrain, position.x, position.z, 0.3) {
                // Get terrain height and spawn 1.0 unit above it
                let terrain_height = minion::terrain::coordinates::get_height_at_world_nearest(
                    terrain, position.x, position.z,
                )
                .unwrap_or(0.0);
                let corrected_position = Vec3::new(position.x, terrain_height + 1.0, position.z);

                let zone = SpawnZone::new(
                    corrected_position,
                    3.0 + (attempts as f32 * 0.1).min(2.0), // Variable radius
                    (2 + (attempts % 3)) as u32,            // Variable enemy count
                    vec!["dark-knight".to_string()],
                )?;
                zones.push(zone);
            }
        }

        if zones.is_empty() {
            // Fallback to non-biome-aware generation
            Self::generate_spawn_zones(terrain, player_spawn, max_zones as u32)
        } else {
            Ok(zones)
        }
    }

    /// Generate biome-aware environment objects with size variation
    fn generate_biome_aware_objects(
        terrain: &TerrainData,
        biome_map: &minion::terrain::biomes::BiomeMap,
        player_spawn: Vec3,
        enemy_zones: &[SpawnZone],
        density: f32,
        object_types: &[String],
        scale_range: (f32, f32),
        seed: u32,
    ) -> MinionResult<Vec<EnvironmentObject>> {
        let mut objects = Vec::new();
        let mut rng = Pcg64::seed_from_u64(seed as u64);

        let constraints = ObjectPlacementConstraints {
            min_distance_from_spawn: 3.0,
            min_distance_from_enemies: 2.0,
            min_distance_between_objects: 1.5,
            max_slope: 0.4,
        };

        let world_width = terrain.width as f32 * terrain.scale;
        let world_height = terrain.height as f32 * terrain.scale;
        let area = world_width * world_height;
        let target_objects = (area * density).round() as usize;

        println!(
            "Placing up to {} biome-aware environment objects (density: {})...",
            target_objects, density
        );

        let mut attempts = 0;
        let max_attempts = target_objects * 10;

        while objects.len() < target_objects && attempts < max_attempts {
            attempts += 1;

            // Generate random world position
            let x = rng.gen_range(-world_width / 2.0..world_width / 2.0);
            let z = rng.gen_range(-world_height / 2.0..world_height / 2.0);

            // Get biome at this location
            let biome_objects = if let Some(blend) = biome_map.get_blend_at_world(x, z) {
                Self::get_biome_appropriate_objects(blend.dominant_biome(), object_types)
            } else {
                object_types.to_vec()
            };

            if biome_objects.is_empty() {
                continue;
            }

            // Select object type based on biome
            let object_type = &biome_objects[rng.gen_range(0..biome_objects.len())];

            // Get biome-specific scale based on object type
            let scale = Self::get_biome_object_scale(
                biome_map
                    .get_blend_at_world(x, z)
                    .and_then(|b| b.dominant_biome()),
                object_type,
                scale_range,
                &mut rng,
            );

            // Use existing object placement validation logic
            if let Some(obj) = Self::try_place_biome_object(
                terrain,
                player_spawn,
                enemy_zones,
                &objects,
                object_type,
                scale,
                &constraints,
                x,
                z,
                &mut rng,
            )? {
                objects.push(obj);
            }
        }

        println!(
            "Placed {} biome-aware environment objects (attempted {} placements)",
            objects.len(),
            attempts
        );

        // Print object summary
        let mut object_counts = std::collections::HashMap::new();
        for obj in &objects {
            *object_counts.entry(&obj.object_type).or_insert(0) += 1;
        }

        if !object_counts.is_empty() {
            println!("  Object types:");
            for (obj_type, count) in object_counts {
                println!("    {}: {} objects", obj_type, count);
            }
        }

        Ok(objects)
    }

    /// Get objects appropriate for a specific biome
    fn get_biome_appropriate_objects(
        biome: Option<BiomeType>,
        available_types: &[String],
    ) -> Vec<String> {
        use BiomeType::*;

        match biome {
            Some(Forest) => {
                // Forest: Lots of trees, some rocks
                let mut objects = Vec::new();
                for obj_type in available_types {
                    match obj_type.as_str() {
                        "tree" => {
                            // 3x more trees in forest
                            objects.extend(vec![obj_type.clone(); 3]);
                        }
                        "rock" => objects.push(obj_type.clone()),
                        _ => objects.push(obj_type.clone()),
                    }
                }
                objects
            }
            Some(Mountains) => {
                // Mountains: Lots of rocks, few trees
                let mut objects = Vec::new();
                for obj_type in available_types {
                    match obj_type.as_str() {
                        "rock" => {
                            // 3x more rocks in mountains
                            objects.extend(vec![obj_type.clone(); 3]);
                        }
                        "tree" => {
                            // Fewer trees in mountains (only if included)
                            if rand::random::<f32>() < 0.3 {
                                objects.push(obj_type.clone());
                            }
                        }
                        _ => objects.push(obj_type.clone()),
                    }
                }
                objects
            }
            Some(Desert) => {
                // Desert: Mostly rocks, no trees
                available_types
                    .iter()
                    .filter(|obj| obj.as_str() != "tree")
                    .cloned()
                    .collect()
            }
            Some(Plains) => {
                // Plains: Balanced mix
                available_types.to_vec()
            }
            Some(Ocean) | Some(Swamp) => {
                // Water biomes: No objects
                Vec::new()
            }
            Some(Tundra) => {
                // Tundra: Some rocks, very few trees
                let mut objects = Vec::new();
                for obj_type in available_types {
                    match obj_type.as_str() {
                        "rock" => objects.push(obj_type.clone()),
                        "tree" => {
                            if rand::random::<f32>() < 0.1 {
                                objects.push(obj_type.clone());
                            }
                        }
                        _ => objects.push(obj_type.clone()),
                    }
                }
                objects
            }
            None => available_types.to_vec(),
        }
    }

    /// Get biome-appropriate scale for objects
    fn get_biome_object_scale(
        biome: Option<BiomeType>,
        object_type: &str,
        base_scale_range: (f32, f32),
        rng: &mut Pcg64,
    ) -> f32 {
        use BiomeType::*;

        let scale_modifier = match (biome, object_type) {
            // Trees
            (Some(Forest), "tree") => rng.gen_range(1.2..2.0), // Large forest trees
            (Some(Mountains), "tree") => rng.gen_range(0.6..1.0), // Smaller mountain trees
            (Some(Plains), "tree") => rng.gen_range(0.8..1.4), // Medium plains trees
            (Some(Tundra), "tree") => rng.gen_range(0.4..0.8), // Small tundra trees

            // Rocks with size distribution
            (Some(Mountains), "rock") => {
                // Mountains have larger rocks with log-normal distribution
                let base = rng.gen_range(0.0..1.0_f32).powf(0.3); // Bias toward larger rocks
                0.8 + base * 2.5 // Range: 0.8 to 3.3 (boulders to massive rocks)
            }
            (Some(Desert), "rock") => {
                // Desert has medium-sized rocks
                let base = rng.gen_range(0.0..1.0_f32).powf(0.5);
                0.6 + base * 1.8 // Range: 0.6 to 2.4
            }
            (Some(Plains), "rock") => {
                // Plains have smaller, scattered rocks
                let base = rng.gen_range(0.0..1.0_f32).powf(0.7);
                0.3 + base * 1.2 // Range: 0.3 to 1.5
            }
            (Some(Forest), "rock") => {
                // Forest has medium rocks
                let base = rng.gen_range(0.0..1.0_f32).powf(0.6);
                0.4 + base * 1.4 // Range: 0.4 to 1.8
            }

            // Default cases
            _ => rng.gen_range(base_scale_range.0..base_scale_range.1),
        };

        scale_modifier
    }

    /// Try to place a biome-specific object at given coordinates
    fn try_place_biome_object(
        terrain: &TerrainData,
        player_spawn: Vec3,
        enemy_zones: &[SpawnZone],
        existing_objects: &[EnvironmentObject],
        object_type: &str,
        scale: f32,
        constraints: &ObjectPlacementConstraints,
        x: f32,
        z: f32,
        rng: &mut Pcg64,
    ) -> MinionResult<Option<EnvironmentObject>> {
        let candidate_pos = Vec3::new(x, 0.0, z);

        // Check distance from player spawn
        if candidate_pos.distance(player_spawn) < constraints.min_distance_from_spawn {
            return Ok(None);
        }

        // Check distance from enemy zones
        for zone in enemy_zones {
            if candidate_pos.distance(zone.center) < constraints.min_distance_from_enemies {
                return Ok(None);
            }
        }

        // Check distance from existing objects
        for obj in existing_objects {
            if candidate_pos.distance(obj.position) < constraints.min_distance_between_objects {
                return Ok(None);
            }
        }

        // Check slope
        if !is_suitable_for_spawning(terrain, x, z, constraints.max_slope) {
            return Ok(None);
        }

        // Get terrain height
        let height = minion::terrain::coordinates::get_height_at_world_interpolated(terrain, x, z)
            .unwrap_or(0.0);

        Ok(Some(EnvironmentObject::new(
            object_type.to_string(),
            Vec3::new(x, height, z),
            Vec3::new(0.0, rng.gen_range(0.0..std::f32::consts::TAU), 0.0),
            Vec3::splat(scale),
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_ring_position() {
        let player_spawn = Vec3::new(32.0, 1.0, 32.0);
        let pos1 = MapGenerator::calculate_ring_position(player_spawn, 0);
        let pos2 = MapGenerator::calculate_ring_position(player_spawn, 1);

        // Positions should be different
        assert_ne!(pos1, pos2);

        // Should be roughly the expected distance from spawn (8.0 to 14.0 based on new logic)
        let distance1 = pos1.distance(player_spawn);
        assert!(distance1 >= 7.0 && distance1 <= 15.0);
    }

    #[test]
    fn test_generate_spawn_zones() {
        let player_spawn = Vec3::new(32.0, 1.0, 32.0);
        let terrain = TerrainData::create_flat(64, 64, 1.0, 0.0).unwrap();

        // Test that the function runs without error, terrain suitability depends on complex logic
        let zones = MapGenerator::generate_spawn_zones(&terrain, player_spawn, 3).unwrap();

        // Test any zones that were created have correct properties
        for zone in &zones {
            assert!(zone.radius >= 3.0);
            assert!(zone.max_enemies >= 2);
            assert_eq!(zone.enemy_types.len(), 1);
            assert_eq!(zone.enemy_types[0], "dark-knight");
        }
    }
}
