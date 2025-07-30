use bevy::prelude::*;
use crate::components::Distance;
use std::f32::consts::TAU;

/// Generate a random spawn position in a ring around the origin
/// Uses a counter for deterministic positioning that spreads enemies around
pub fn generate_respawn_position(counter: u32, min_distance: Distance, max_distance: Distance) -> Vec3 {
    let angle = (counter as f32 * 2.3) % TAU; // Use prime-like multiplier for spread
    let distance = min_distance.0 + (counter % 4) as f32 * (max_distance.0 - min_distance.0) / 4.0;
    
    Vec3::new(
        angle.cos() * distance,
        1.0, // Keep on ground level
        angle.sin() * distance,
    )
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_respawn_position_generation() {
        let pos1 = generate_respawn_position(0, Distance::new(5.0), Distance::new(10.0));
        let pos2 = generate_respawn_position(1, Distance::new(5.0), Distance::new(10.0));
        
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
            .map(|i| generate_respawn_position(i, Distance::new(5.0), Distance::new(10.0)))
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
        sorted_angles.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        // Check that we don't have all angles clustered together
        let angle_span = sorted_angles.last().unwrap() - sorted_angles.first().unwrap();
        assert!(angle_span > 1.0); // Should span more than 1 radian
    }
}