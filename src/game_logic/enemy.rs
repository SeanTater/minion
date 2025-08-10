use crate::map::SpawnZone;
use bevy::prelude::*;
use std::f32::consts::TAU;

/// Flocking force calculation result
#[derive(Debug, Clone, PartialEq)]
pub struct FlockingForces {
    /// Separation force to avoid other enemies
    pub separation: Vec3,
    /// Combined flocking force
    pub total_force: Vec3,
}

/// Enemy spawn position calculation using deterministic ring-based placement
///
/// Uses a prime-like multiplier (2.3) for angular distribution and square root
/// for radial distribution to create natural-looking enemy placement patterns.
///
/// # Arguments
/// * `spawn_zone` - The zone to spawn enemies within
/// * `enemy_index` - Index of the enemy being spawned (0-based)
///
/// # Returns
/// World position for the enemy spawn location
pub fn calculate_enemy_spawn_position(spawn_zone: &SpawnZone, enemy_index: u32) -> Vec3 {
    // Use prime-like multiplier for good angular distribution
    let angle = (enemy_index as f32 * 2.3) % TAU;

    // Use square root for more natural radial distribution (more enemies near edge)
    let distance_factor = (enemy_index as f32 / spawn_zone.max_enemies as f32).sqrt();
    let distance = spawn_zone.radius * (0.3 + 0.7 * distance_factor);

    Vec3::new(
        spawn_zone.center.x + angle.cos() * distance,
        spawn_zone.center.y + 1.0, // Spawn above terrain for character controller
        spawn_zone.center.z + angle.sin() * distance,
    )
}

/// Calculate flocking separation forces to prevent enemy clustering
///
/// Implements a two-pass flocking system where separation forces are calculated
/// based on distance to other enemies. Closer enemies generate stronger repulsion.
///
/// # Arguments
/// * `enemy_position` - Current enemy position (2D, Y ignored)
/// * `other_positions` - Positions of all other enemies
/// * `separation_radius` - Maximum distance for separation effects
///
/// # Returns
/// Flocking forces including separation and total combined force
pub fn calculate_flocking_forces(
    enemy_position: Vec3,
    other_positions: &[(Vec3, bool)], // (position, is_dying)
    separation_radius: f32,
) -> FlockingForces {
    let mut separation_force = Vec3::ZERO;

    for (other_pos, is_dying) in other_positions {
        if *is_dying {
            continue; // Skip dying enemies
        }

        let distance = enemy_position.distance(*other_pos);

        if distance < separation_radius && distance > 0.1 {
            let away_from_other = (enemy_position - *other_pos).normalize();
            let separation_strength = (separation_radius - distance) / separation_radius;
            separation_force += away_from_other * separation_strength;
        }
    }

    FlockingForces {
        separation: separation_force,
        total_force: separation_force, // Currently only separation, could add cohesion/alignment
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enemy_spawn_position_deterministic() {
        let spawn_zone = SpawnZone {
            center: Vec3::new(10.0, 0.0, 10.0),
            radius: 5.0,
            max_enemies: 8,
            enemy_types: vec!["dark-knight".to_string()],
        };

        let pos1 = calculate_enemy_spawn_position(&spawn_zone, 0);
        let pos2 = calculate_enemy_spawn_position(&spawn_zone, 0);

        // Same input should produce same output
        assert_eq!(pos1, pos2);

        // Different indices should produce different positions
        let pos3 = calculate_enemy_spawn_position(&spawn_zone, 1);
        assert_ne!(pos1, pos3);
    }

    #[test]
    fn test_enemy_spawn_position_within_bounds() {
        let spawn_zone = SpawnZone {
            center: Vec3::new(0.0, 0.0, 0.0),
            radius: 10.0,
            max_enemies: 20,
            enemy_types: vec!["dark-knight".to_string()],
        };

        for i in 0..20 {
            let pos = calculate_enemy_spawn_position(&spawn_zone, i);

            // Should be at correct height
            assert_eq!(pos.y, 1.0);

            // Should be within spawn zone radius
            let distance_from_center = Vec3::new(pos.x, 0.0, pos.z).length();
            assert!(distance_from_center <= spawn_zone.radius);

            // Should maintain minimum distance (30% of radius)
            assert!(distance_from_center >= spawn_zone.radius * 0.3);
        }
    }

    #[test]
    fn test_enemy_spawn_position_distribution() {
        let spawn_zone = SpawnZone {
            center: Vec3::new(0.0, 0.0, 0.0),
            radius: 10.0,
            max_enemies: 16,
            enemy_types: vec!["dark-knight".to_string()],
        };

        let positions: Vec<Vec3> = (0..16)
            .map(|i| calculate_enemy_spawn_position(&spawn_zone, i))
            .collect();

        // All positions should be unique
        for i in 0..positions.len() {
            for j in (i + 1)..positions.len() {
                assert_ne!(positions[i], positions[j]);
            }
        }

        // Check angular distribution
        let angles: Vec<f32> = positions.iter().map(|pos| pos.z.atan2(pos.x)).collect();

        let mut sorted_angles = angles.clone();
        sorted_angles.sort_by(|a, b| a.partial_cmp(b).unwrap());

        // Should have good angular spread
        if let (Some(first), Some(last)) = (sorted_angles.first(), sorted_angles.last()) {
            let angle_span = last - first;
            assert!(angle_span > 2.0); // Should span more than 2 radians
        }
    }

    #[test]
    fn test_flocking_forces_no_neighbors() {
        let enemy_pos = Vec3::new(0.0, 0.0, 0.0);
        let other_positions = vec![];

        let forces = calculate_flocking_forces(enemy_pos, &other_positions, 2.0);

        assert_eq!(forces.separation, Vec3::ZERO);
        assert_eq!(forces.total_force, Vec3::ZERO);
    }

    #[test]
    fn test_flocking_forces_single_neighbor() {
        let enemy_pos = Vec3::new(0.0, 0.0, 0.0);
        let other_positions = vec![(Vec3::new(1.0, 0.0, 0.0), false)];

        let forces = calculate_flocking_forces(enemy_pos, &other_positions, 2.0);

        // Should push away from the neighbor
        assert!(forces.separation.x < 0.0); // Away from positive X neighbor
        assert_eq!(forces.separation.y, 0.0);
        assert_eq!(forces.separation.z, 0.0);
        assert!(forces.separation.length() > 0.0);
    }

    #[test]
    fn test_flocking_forces_dying_enemies_ignored() {
        let enemy_pos = Vec3::new(0.0, 0.0, 0.0);
        let other_positions = vec![
            (Vec3::new(1.0, 0.0, 0.0), true), // Dying enemy - should be ignored
            (Vec3::new(-1.0, 0.0, 0.0), false), // Living enemy
        ];

        let forces = calculate_flocking_forces(enemy_pos, &other_positions, 2.0);

        // Should only be affected by living enemy at (-1, 0, 0)
        assert!(forces.separation.x > 0.0); // Away from negative X neighbor
        assert_eq!(forces.separation.z, 0.0);
    }

    #[test]
    fn test_flocking_forces_distance_falloff() {
        let enemy_pos = Vec3::new(0.0, 0.0, 0.0);
        let separation_radius = 2.0;

        // Close neighbor
        let close_neighbor = vec![(Vec3::new(0.5, 0.0, 0.0), false)];
        let close_forces = calculate_flocking_forces(enemy_pos, &close_neighbor, separation_radius);

        // Far neighbor
        let far_neighbor = vec![(Vec3::new(1.5, 0.0, 0.0), false)];
        let far_forces = calculate_flocking_forces(enemy_pos, &far_neighbor, separation_radius);

        // Closer neighbor should generate stronger force
        assert!(close_forces.separation.length() > far_forces.separation.length());
    }

    #[test]
    fn test_flocking_forces_multiple_neighbors() {
        let enemy_pos = Vec3::new(0.0, 0.0, 0.0);
        let other_positions = vec![
            (Vec3::new(1.0, 0.0, 0.0), false),  // Right
            (Vec3::new(-1.0, 0.0, 0.0), false), // Left
            (Vec3::new(0.0, 0.0, 1.0), false),  // Forward
            (Vec3::new(0.0, 0.0, -1.0), false), // Back
        ];

        let forces = calculate_flocking_forces(enemy_pos, &other_positions, 2.0);

        // Forces should roughly cancel out in X and Z due to symmetry
        assert!(forces.separation.x.abs() < 0.1);
        assert!(forces.separation.z.abs() < 0.1);
        assert_eq!(forces.separation.y, 0.0);
    }

    // Property-based tests for mathematical invariants
    #[test]
    fn test_spawn_position_invariants() {
        let spawn_zone = SpawnZone {
            center: Vec3::new(5.0, 2.0, -3.0),
            radius: 8.0,
            max_enemies: 50,
            enemy_types: vec!["dark-knight".to_string()],
        };

        for i in 0..50 {
            let pos = calculate_enemy_spawn_position(&spawn_zone, i);

            // Invariant: Y coordinate should always be center.y + 1.0
            assert_eq!(pos.y, spawn_zone.center.y + 1.0);

            // Invariant: Distance from center should be within bounds
            let distance_2d = Vec3::new(
                pos.x - spawn_zone.center.x,
                0.0,
                pos.z - spawn_zone.center.z,
            )
            .length();

            assert!(distance_2d >= spawn_zone.radius * 0.3);
            assert!(distance_2d <= spawn_zone.radius);
        }
    }

    #[test]
    fn test_flocking_forces_invariants() {
        let enemy_pos = Vec3::new(0.0, 0.0, 0.0);
        let separation_radius = 3.0;

        // Test with various configurations
        let test_cases = vec![
            vec![],                                  // No neighbors
            vec![(Vec3::new(1.0, 0.0, 0.0), false)], // Single neighbor
            vec![(Vec3::new(1.0, 0.0, 0.0), true)],  // Single dying neighbor
            vec![
                // Multiple neighbors
                (Vec3::new(1.0, 0.0, 0.0), false),
                (Vec3::new(0.0, 0.0, 1.0), false),
                (Vec3::new(-1.0, 0.0, 0.0), true), // Dying
            ],
        ];

        for other_positions in test_cases {
            let forces = calculate_flocking_forces(enemy_pos, &other_positions, separation_radius);

            // Invariant: Y component should always be zero (2D movement)
            assert_eq!(forces.separation.y, 0.0);
            assert_eq!(forces.total_force.y, 0.0);

            // Invariant: Forces should be finite
            assert!(forces.separation.is_finite());
            assert!(forces.total_force.is_finite());

            // Invariant: Total force should equal separation (currently no other forces)
            assert_eq!(forces.total_force, forces.separation);
        }
    }
}
