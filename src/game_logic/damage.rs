use bevy::prelude::*;

/// Calculate area effect damage with distance-based falloff
/// Returns None if target is outside the effect radius
pub fn calculate_area_damage(
    base_damage_per_second: i32,
    delta_time: f32,
    target_position: Vec3,
    effect_position: Vec3,
    radius: f32,
) -> Option<i32> {
    let distance = target_position.distance(effect_position);
    
    if distance > radius {
        return None;
    }
    
    // Distance-based damage falloff
    let damage_multiplier = (1.0 - (distance / radius)).max(0.0);
    let damage_float = base_damage_per_second as f32 * damage_multiplier * delta_time;
    
    if damage_float > 0.0 {
        Some((damage_float as i32).max(1)) // Ensure at least 1 damage if any is dealt
    } else {
        None
    }
}

/// Check if two entities are within collision distance
pub fn check_collision(pos1: Vec3, pos2: Vec3, collision_distance: f32) -> bool {
    pos1.distance(pos2) <= collision_distance
}

/// Calculate pushback force between two entities that are too close
pub fn calculate_pushback(
    pos1: Vec3,
    pos2: Vec3,
    min_distance: f32,
) -> Option<(Vec3, Vec3)> {
    let distance = pos1.distance(pos2);
    
    if distance < min_distance && distance > 0.0 {
        let pushback_force = (min_distance - distance) * 0.5;
        let direction = (pos1 - pos2).normalize();
        
        let push1 = direction * pushback_force * 0.5;
        let push2 = -direction * pushback_force * 0.5;
        
        Some((push1, push2))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_area_damage_direct_hit() {
        let damage = calculate_area_damage(
            100, // 100 DPS
            0.016, // ~60fps delta time
            Vec3::ZERO,
            Vec3::ZERO,
            3.0,
        );
        
        assert_eq!(damage, Some(1)); // 100 * 1.0 * 0.016 = 1.6 -> 1
    }

    #[test]
    fn test_area_damage_edge_hit() {
        let damage = calculate_area_damage(
            100,
            0.016,
            Vec3::new(3.0, 0.0, 0.0), // At edge of radius
            Vec3::ZERO,
            3.0,
        );
        
        assert_eq!(damage, None); // Should be outside radius
    }

    #[test]
    fn test_area_damage_partial_hit() {
        let damage = calculate_area_damage(
            100,
            0.016,
            Vec3::new(1.5, 0.0, 0.0), // Half distance
            Vec3::ZERO,
            3.0,
        );
        
        // Distance = 1.5, radius = 3.0
        // Multiplier = 1.0 - (1.5/3.0) = 0.5
        // Damage = 100 * 0.5 * 0.016 = 0.8 -> 1 (minimum)
        assert_eq!(damage, Some(1));
    }

    #[test]
    fn test_area_damage_outside_radius() {
        let damage = calculate_area_damage(
            100,
            0.016,
            Vec3::new(5.0, 0.0, 0.0),
            Vec3::ZERO,
            3.0,
        );
        
        assert_eq!(damage, None);
    }

    #[test]
    fn test_collision_detection() {
        assert!(check_collision(Vec3::ZERO, Vec3::new(0.5, 0.0, 0.0), 0.6));
        assert!(!check_collision(Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0), 0.6));
    }

    #[test]
    fn test_pushback_calculation() {
        let (push1, push2) = calculate_pushback(
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::ZERO,
            1.5,
        ).unwrap();
        
        // Distance = 1.0, min_distance = 1.5
        // Pushback force = (1.5 - 1.0) * 0.5 = 0.25
        // Direction from pos2 to pos1 = (1, 0, 0)
        // push1 should be positive direction, push2 negative
        assert!(push1.x > 0.0);
        assert!(push2.x < 0.0);
        assert_eq!(push1.x, -push2.x); // Equal and opposite
    }

    #[test]
    fn test_no_pushback_when_far_enough() {
        let result = calculate_pushback(
            Vec3::new(2.0, 0.0, 0.0),
            Vec3::ZERO,
            1.5,
        );
        
        assert_eq!(result, None);
    }
}