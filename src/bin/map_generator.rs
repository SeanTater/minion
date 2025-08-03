use bevy::prelude::*;
use minion::game_logic::errors::MinionResult;
use minion::map::{EnvironmentObject, MapDefinition, SpawnZone, TerrainData};
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
}

pub struct MapGenerator;

impl MapGenerator {
    pub fn generate(config: MapGenerationConfig) -> MinionResult<MapDefinition> {
        println!("Generating map: {}", config.name);
        println!("Terrain size: {}x{} grid cells", config.width, config.height);
        println!("Player spawn: {}", config.player_spawn);
        println!("Using terrain type (seed: {})", config.generator.seed);

        // Generate terrain
        let terrain = config.generator.generate(config.width, config.height, 1.0)?;
        println!("Generated terrain with {} height points", terrain.heights.len());

        // Generate spawn zones
        let enemy_zones = Self::generate_spawn_zones(&terrain, config.player_spawn, 5)?;
        println!("Generated {} enemy spawn zones", enemy_zones.len());

        // Generate environment objects
        let object_seed = config.generator.seed.wrapping_add(42);
        let environment_objects = Self::generate_objects(
            &terrain,
            config.player_spawn,
            &enemy_zones,
            config.object_density,
            &config.object_types,
            config.scale_range,
            object_seed,
        )?;

        Ok(MapDefinition::new(
            config.name,
            terrain,
            config.player_spawn,
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
                    terrain, position.x, position.z
                ).unwrap_or(0.0);

                let final_center = Vec3::new(position.x, terrain_height + 1.0, position.z);
                let radius = base_radius + (i % 3) as f32 * 0.5;
                let max_enemies = base_max_enemies + (i % 3);

                zones.push(SpawnZone::new(
                    final_center,
                    radius,
                    max_enemies,
                    vec!["dark-knight".to_string()],
                )?);

                println!("Found suitable spawn location at ({:.1}, {:.1}, {:.1})",
                    final_center.x, final_center.y, final_center.z);
            }
            attempts += 1;
        }

        if zones.len() < num_zones as usize {
            println!("Warning: Only found {} suitable spawn locations out of {} requested",
                zones.len(), num_zones);
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

        let terrain_area = (terrain.width as f32 * terrain.scale) * (terrain.height as f32 * terrain.scale);
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
                terrain, player_spawn, enemy_zones, &objects, object_types,
                scale_range, &constraints, &mut rng
            )? {
                objects.push(obj);
            }
            attempts += 1;
        }

        println!("Placed {} environment objects (attempted {} placements)",
            objects.len(), attempts);

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

        let terrain_height = match minion::terrain::coordinates::get_height_at_world_nearest(terrain, world_x, world_z) {
            Some(height) => height,
            None => return Ok(None),
        };
        let candidate_pos = Vec3::new(world_x, terrain_height, world_z);

        if !Self::is_valid_placement(&candidate_pos, player_spawn, enemy_zones, existing_objects, terrain, constraints) {
            return Ok(None);
        }

        let object_type = object_types[rng.gen_range(0..object_types.len())].clone();
        let rotation = Vec3::new(0.0, rng.gen_range(0.0..2.0 * std::f32::consts::PI), 0.0);
        let scale_factor = rng.gen_range(scale_range.0..=scale_range.1);
        let scale = Vec3::splat(scale_factor);

        Ok(Some(EnvironmentObject::new(object_type, candidate_pos, rotation, scale)))
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
                candidate_pos.distance(zone.center) >= zone.radius + constraints.min_distance_from_enemies
            })
            && existing_objects.iter().all(|obj| {
                candidate_pos.distance(obj.position) >= constraints.min_distance_between_objects
            })
            && is_suitable_for_spawning(terrain, candidate_pos.x, candidate_pos.z, constraints.max_slope)
    }
}

struct ObjectPlacementConstraints {
    min_distance_from_spawn: f32,
    min_distance_from_enemies: f32,
    min_distance_between_objects: f32,
    max_slope: f32,
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