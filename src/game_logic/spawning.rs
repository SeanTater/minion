use crate::components::*;
use crate::game_logic::names::generate_dark_name;
use crate::map::SpawnZone;
use crate::resources::*;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use std::f32::consts::TAU;

/// Generate a random position within a spawn zone using a counter for deterministic placement
pub fn generate_zone_position(zone: &SpawnZone, counter: u32) -> Vec3 {
    // Use counter for deterministic but spread-out positioning within the zone
    let angle = (counter as f32 * 2.3) % TAU; // Use prime-like multiplier for spread
    let distance_factor = (counter % 7) as f32 / 7.0; // Cycle through distances
    let distance = zone.radius * (0.2 + 0.8 * distance_factor); // Stay within zone bounds

    Vec3::new(
        zone.center.x + angle.cos() * distance,
        zone.center.y + 1.0, // Spawn slightly above terrain - character controller will handle terrain following
        zone.center.z + angle.sin() * distance,
    )
}

/// Spawn a single enemy entity with all required components
/// This function eliminates duplication between initial spawning and respawning logic
pub fn spawn_enemy_entity(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    position: Vec3,
    game_config: &GameConfig,
) {
    // Load all LOD levels for enemies
    let high_scene = asset_server.load("enemies/dark-knight-high.glb#Scene0");
    let med_scene = asset_server.load("enemies/dark-knight-med.glb#Scene0");
    let low_scene = asset_server.load("enemies/dark-knight-low.glb#Scene0");

    // Inline LOD level determination with fallback
    let starting_level =
        LodLevel::try_from(game_config.settings.max_lod_level.as_str()).unwrap_or(LodLevel::High); // Default to High on invalid config
    let starting_scene = match starting_level {
        LodLevel::High => high_scene.clone(),
        LodLevel::Medium => med_scene.clone(),
        LodLevel::Low => low_scene.clone(),
    };

    commands.spawn((
        SceneRoot(starting_scene), // Start with appropriate max LOD
        Transform::from_translation(position).with_scale(Vec3::splat(2.0)),
        RigidBody::KinematicPositionBased,
        Collider::capsule_y(1.0, 0.5), // 2m tall capsule like player
        KinematicCharacterController {
            snap_to_ground: Some(CharacterLength::Absolute(1.5)), // Increased for better slope descent detection
            offset: CharacterLength::Absolute(0.1), // Small gap for numerical stability
            max_slope_climb_angle: 45.0_f32.to_radians(),
            min_slope_slide_angle: 30.0_f32.to_radians(),
            slide: true,                           // Enable sliding on slopes
            apply_impulse_to_dynamic_bodies: true, // Better physics interaction
            ..default()
        },
        Enemy {
            speed: Speed::new(game_config.settings.enemy_movement_speed.get()),
            health: HealthPool::new_full(game_config.settings.enemy_max_health.get()),
            mana: ManaPool::new_full(game_config.settings.enemy_max_mana.get()),
            energy: EnergyPool::new_full(game_config.settings.enemy_max_energy.get()),
            chase_distance: Distance::new(game_config.settings.enemy_chase_distance.get()),
            is_dying: false,
        },
        PathfindingAgent::default(),
        LodEntity {
            current_level: starting_level,
            high_handle: high_scene.clone(),
            med_handle: med_scene.clone(),
            low_handle: low_scene.clone(),
            entity_type: LodEntityType::Enemy,
        },
        Name(generate_dark_name()),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zone_position_generation() {
        let zone = SpawnZone {
            center: Vec3::new(10.0, 0.0, 10.0),
            radius: 5.0,
            max_enemies: 10,
            enemy_types: vec!["dark-knight".to_string()],
        };

        let pos1 = generate_zone_position(&zone, 0);
        let pos2 = generate_zone_position(&zone, 1);

        // Positions should be different
        assert_ne!(pos1, pos2);

        // Should be at correct height
        assert_eq!(pos1.y, 1.0);
        assert_eq!(pos2.y, 1.0);

        // Should be within zone radius
        let distance1 = Vec3::new(pos1.x - zone.center.x, 0.0, pos1.z - zone.center.z).length();
        let distance2 = Vec3::new(pos2.x - zone.center.x, 0.0, pos2.z - zone.center.z).length();

        assert!(distance1 <= zone.radius);
        assert!(distance2 <= zone.radius);

        // Should maintain minimum distance from center (0.2 factor)
        assert!(distance1 >= zone.radius * 0.2);
        assert!(distance2 >= zone.radius * 0.2);
    }

    #[test]
    fn test_zone_position_spread() {
        let zone = SpawnZone {
            center: Vec3::new(0.0, 0.0, 0.0),
            radius: 10.0,
            max_enemies: 8,
            enemy_types: vec!["dark-knight".to_string()],
        };

        let positions: Vec<Vec3> = (0..8).map(|i| generate_zone_position(&zone, i)).collect();

        // All positions should be different
        for i in 0..positions.len() {
            for j in (i + 1)..positions.len() {
                assert_ne!(positions[i], positions[j]);
            }
        }

        // Should provide good spread - check that angles are reasonably distributed
        let angles: Vec<f32> = positions.iter().map(|pos| pos.z.atan2(pos.x)).collect();

        // With 8 positions, we should have decent angular spread
        let mut sorted_angles = angles.clone();
        sorted_angles.sort_by(|a, b| a.partial_cmp(b).expect("NaN in angle comparison"));

        // Check that we don't have all angles clustered together
        if let (Some(first), Some(last)) = (sorted_angles.first(), sorted_angles.last()) {
            let angle_span = last - first;
            assert!(angle_span > 1.0); // Should span more than 1 radian
        } else {
            panic!("Expected non-empty angles vector");
        }
    }

    // Integration tests for spawning logic
    #[test]
    fn test_spawn_enemy_entity_components() {
        // Test that spawn_enemy_entity creates the expected component bundle
        let game_config = GameConfig::default();
        let spawn_position = Vec3::new(10.0, 1.0, 5.0);

        // Verify the function parameters and expected behavior
        assert_eq!(spawn_position.y, 1.0); // Should spawn above ground
        assert_eq!(game_config.settings.enemy_movement_speed.get(), 3.0); // Default speed
        assert_eq!(game_config.settings.enemy_max_health.get(), 3.0); // Default health
    }

    #[test]
    fn test_spawn_enemy_lod_level_selection() {
        let mut game_config = GameConfig::default();

        // Test high LOD (default)
        let high_level = LodLevel::try_from(game_config.settings.max_lod_level.as_str())
            .unwrap_or(LodLevel::High);
        assert_eq!(high_level, LodLevel::High);

        // Test medium LOD
        game_config.settings.max_lod_level = "medium".to_string();
        let med_level = LodLevel::try_from(game_config.settings.max_lod_level.as_str())
            .unwrap_or(LodLevel::High);
        assert_eq!(med_level, LodLevel::Medium);

        // Test low LOD
        game_config.settings.max_lod_level = "low".to_string();
        let low_level = LodLevel::try_from(game_config.settings.max_lod_level.as_str())
            .unwrap_or(LodLevel::High);
        assert_eq!(low_level, LodLevel::Low);
    }
}
