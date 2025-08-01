use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::game_logic::{errors::{MinionError, MinionResult}, names::generate_dark_name};
use std::f32::consts::TAU;

/// Generate a random spawn position in a ring around the origin
/// Uses a counter for deterministic positioning that spreads enemies around
pub fn generate_respawn_position(counter: u32, min_distance: Distance, max_distance: Distance) -> MinionResult<Vec3> {
    if min_distance.0 >= max_distance.0 {
        return Err(MinionError::InvalidSpawnPosition { 
            position: Vec3::ZERO 
        });
    }
    
    let angle = (counter as f32 * 2.3) % TAU; // Use prime-like multiplier for spread
    let distance = min_distance.0 + (counter % 4) as f32 * (max_distance.0 - min_distance.0) / 4.0;
    
    let position = Vec3::new(
        angle.cos() * distance,
        1.0, // Keep on ground level
        angle.sin() * distance,
    );
    
    Ok(position)
}

/// Generate a random spawn position in a ring around the origin (fallback version)
/// Uses a counter for deterministic positioning that spreads enemies around
pub fn generate_respawn_position_unchecked(counter: u32, min_distance: Distance, max_distance: Distance) -> Vec3 {
    generate_respawn_position(counter, min_distance, max_distance)
        .unwrap_or_else(|err| {
            eprintln!("Warning: Spawn position generation failed ({}), using fallback", err);
            Vec3::new(5.0, 1.0, 0.0) // Safe fallback position
        })
}

/// Check if a position is valid for spawning (not too close to other entities)
pub fn is_valid_spawn_position(
    position: Vec3,
    existing_positions: &[Vec3],
    min_distance: Distance,
) -> bool {
    for &existing_pos in existing_positions {
        if position.distance(existing_pos) < min_distance.0 {
            return false;
        }
    }
    true
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

    // Determine starting LOD level based on global max setting
    let (starting_scene, starting_level) = match game_config.settings.max_lod_level.as_str() {
        "medium" => (med_scene.clone(), LodLevel::Medium),
        "low" => (low_scene.clone(), LodLevel::Low),
        _ => (high_scene.clone(), LodLevel::High),
    };

    commands.spawn((
        SceneRoot(starting_scene), // Start with appropriate max LOD
        Transform::from_translation(position)
            .with_scale(Vec3::splat(2.0))
            .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
        RigidBody::Dynamic,
        Collider::capsule_y(1.0, 0.5), // 2m tall capsule like player
        LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z, // Prevent tipping over
        Friction::coefficient(0.7),
        Restitution::coefficient(0.0), // No bouncing
        ColliderMassProperties::Density(0.8), // Slightly lighter than player
        ExternalForce::default(),
        Velocity::default(),
        Damping { linear_damping: 3.0, angular_damping: 8.0 }, // Add damping for more realistic movement
        Enemy {
            speed: Speed::new(game_config.settings.enemy_movement_speed),
            health: HealthPool::new_full(game_config.settings.enemy_max_health),
            mana: ManaPool::new_full(game_config.settings.enemy_max_mana),
            energy: EnergyPool::new_full(game_config.settings.enemy_max_energy),
            chase_distance: Distance::new(game_config.settings.enemy_chase_distance),
            is_dying: false,
        },
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
    fn test_respawn_position_generation() {
        let pos1 = generate_respawn_position(0, Distance::new(5.0), Distance::new(10.0)).unwrap();
        let pos2 = generate_respawn_position(1, Distance::new(5.0), Distance::new(10.0)).unwrap();
        
        // Positions should be different
        assert_ne!(pos1, pos2);
        
        // Should be on ground level
        assert_eq!(pos1.y, 1.0);
        assert_eq!(pos2.y, 1.0);
        
        // Should be within distance range
        let distance1 = Vec3::new(pos1.x, 0.0, pos1.z).length();
        let distance2 = Vec3::new(pos2.x, 0.0, pos2.z).length();
        
        assert!(distance1 >= 5.0 && distance1 <= 10.0);
        assert!(distance2 >= 5.0 && distance2 <= 10.0);
    }

    #[test]
    fn test_spawn_position_validation() {
        let position = Vec3::new(5.0, 1.0, 0.0);
        let existing = vec![
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(10.0, 1.0, 0.0),
        ];
        
        // Should be valid - far enough from existing positions  
        assert!(is_valid_spawn_position(position, &existing, Distance::new(2.0)));
        
        // Should be invalid - too close to first position
        assert!(!is_valid_spawn_position(position, &existing, Distance::new(6.0)));
    }

    #[test]
    fn test_counter_spread() {
        let positions: Vec<Vec3> = (0..8)
            .map(|i| generate_respawn_position(i, Distance::new(5.0), Distance::new(10.0)).unwrap())
            .collect();
        
        // All positions should be different
        for i in 0..positions.len() {
            for j in (i + 1)..positions.len() {
                assert_ne!(positions[i], positions[j]);
            }
        }
        
        // Should provide good spread - check that angles are reasonably distributed
        let angles: Vec<f32> = positions
            .iter()
            .map(|pos| pos.z.atan2(pos.x))
            .collect();
        
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

    #[test]
    fn test_invalid_spawn_parameters() {
        // min_distance >= max_distance should return error
        let result = generate_respawn_position(0, Distance::new(10.0), Distance::new(5.0));
        assert!(result.is_err());
        
        let result = generate_respawn_position(0, Distance::new(5.0), Distance::new(5.0));
        assert!(result.is_err());
        
        // Fallback function should handle errors gracefully
        let pos = generate_respawn_position_unchecked(0, Distance::new(10.0), Distance::new(5.0));
        assert_eq!(pos, Vec3::new(5.0, 1.0, 0.0)); // Fallback position
    }
}