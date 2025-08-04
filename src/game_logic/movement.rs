use bevy::prelude::*;
// Removed unused imports

/// Pure movement calculation logic that can be tested without Bevy runtime
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MovementCalculation {
    pub movement_vector: Vec3,
    pub should_move: bool,
    pub distance_to_target: f32,
    pub slowdown_factor: f32,
    pub rotation_target: Option<Vec3>,
}

/// Configuration for movement calculations
#[derive(Debug, Clone, Copy)]
pub struct MovementConfig {
    pub speed: f32,
    pub stopping_distance: f32,
    pub slowdown_distance: f32,
    pub delta_time: f32,
}

impl Default for MovementConfig {
    fn default() -> Self {
        Self {
            speed: 5.0,
            stopping_distance: 0.1,
            slowdown_distance: 2.0,
            delta_time: 1.0 / 60.0, // 60 FPS
        }
    }
}

/// Calculate movement for a character given current position, target, and configuration
pub fn calculate_movement(
    current_position: Vec3,
    target_position: Option<Vec3>,
    config: MovementConfig,
) -> MovementCalculation {
    let Some(target) = target_position else {
        return MovementCalculation {
            movement_vector: Vec3::ZERO,
            should_move: false,
            distance_to_target: 0.0,
            slowdown_factor: 0.0,
            rotation_target: None,
        };
    };

    // Calculate 2D distance (ignore Y differences for movement)
    let current_2d = Vec3::new(current_position.x, 0.0, current_position.z);
    let target_2d = Vec3::new(target.x, 0.0, target.z);
    let direction = (target_2d - current_2d).normalize_or_zero();
    let distance = current_2d.distance(target_2d);

    // Check if we should stop
    if distance <= config.stopping_distance {
        return MovementCalculation {
            movement_vector: Vec3::ZERO,
            should_move: false,
            distance_to_target: distance,
            slowdown_factor: 0.0,
            rotation_target: None,
        };
    }

    // Calculate movement
    let max_move_distance = config.speed * config.delta_time;

    // Apply slowdown as we approach target
    let slowdown_factor = (distance / config.slowdown_distance).min(1.0);
    let actual_move_distance = max_move_distance * slowdown_factor;

    // Clamp movement to not overshoot target
    let clamped_move_distance = actual_move_distance.min(distance);
    let movement_vector = direction * clamped_move_distance;

    // Calculate rotation target (for GLB models facing backwards)
    let rotation_target = if direction.length() > 0.1 {
        Some(Vec3::new(
            current_position.x - direction.x, // Flip for GLB orientation
            current_position.y,               // Keep same Y level
            current_position.z - direction.z, // Flip for GLB orientation
        ))
    } else {
        None
    };

    MovementCalculation {
        movement_vector,
        should_move: true,
        distance_to_target: distance,
        slowdown_factor,
        rotation_target,
    }
}

/// Validate a target position for movement
pub fn validate_target(current_position: Vec3, target_position: Vec3) -> bool {
    let distance = current_position.distance(target_position);

    // Basic validation rules
    distance > 0.01 && // Must be meaningful distance
    distance < 1000.0 && // Reasonable maximum distance
    target_position.y.is_finite() && // Must be valid coordinates
    target_position.x.is_finite() &&
    target_position.z.is_finite()
}

/// Convert click ray to ground target position
pub fn ray_to_ground_target(ray_origin: Vec3, ray_direction: Vec3, ground_y: f32) -> Option<Vec3> {
    if ray_direction.y.abs() < 0.001 {
        return None; // Ray is parallel to ground
    }

    let t = (ground_y - ray_origin.y) / ray_direction.y;
    if t < 0.0 {
        return None; // Ray pointing away from ground
    }

    let hit_point = ray_origin + ray_direction * t;
    Some(Vec3::new(hit_point.x, ground_y, hit_point.z))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_movement_calculation() {
        let current = Vec3::new(0.0, 1.0, 0.0);
        let target = Vec3::new(5.0, 1.0, 0.0);
        let config = MovementConfig::default();

        let result = calculate_movement(current, Some(target), config);

        assert!(result.should_move);
        assert_eq!(result.distance_to_target, 5.0);
        assert!(result.movement_vector.length() > 0.0);
        assert!(result.movement_vector.x > 0.0); // Moving in positive X direction
        assert_eq!(result.movement_vector.y, 0.0); // No Y movement
    }

    #[test]
    fn test_no_target_no_movement() {
        let current = Vec3::new(0.0, 1.0, 0.0);
        let config = MovementConfig::default();

        let result = calculate_movement(current, None, config);

        assert!(!result.should_move);
        assert_eq!(result.movement_vector, Vec3::ZERO);
        assert_eq!(result.distance_to_target, 0.0);
    }

    #[test]
    fn test_close_target_no_movement() {
        let current = Vec3::new(0.0, 1.0, 0.0);
        let target = Vec3::new(0.05, 1.0, 0.0); // Within stopping distance
        let config = MovementConfig::default();

        let result = calculate_movement(current, Some(target), config);

        assert!(!result.should_move);
        assert_eq!(result.movement_vector, Vec3::ZERO);
    }

    #[test]
    fn test_slowdown_near_target() {
        let current = Vec3::new(0.0, 1.0, 0.0);
        let target = Vec3::new(1.0, 1.0, 0.0); // Within slowdown distance (2.0)
        let config = MovementConfig::default();

        let result = calculate_movement(current, Some(target), config);

        assert!(result.should_move);
        assert!(result.slowdown_factor < 1.0);
        assert!(result.movement_vector.length() < config.speed * config.delta_time);
    }

    #[test]
    fn test_movement_ignores_y_differences() {
        let current = Vec3::new(0.0, 1.0, 0.0);
        let target = Vec3::new(3.0, 5.0, 4.0); // Different Y level
        let config = MovementConfig::default();

        let result = calculate_movement(current, Some(target), config);

        assert!(result.should_move);
        // Movement should only be in X and Z
        assert_eq!(result.movement_vector.y, 0.0);
        // Distance calculation should ignore Y
        assert_eq!(result.distance_to_target, 5.0); // sqrt(3^2 + 4^2) = 5
    }

    #[test]
    fn test_movement_clamping() {
        let current = Vec3::new(0.0, 1.0, 0.0);
        let target = Vec3::new(0.5, 1.0, 0.0); // Close but outside stopping distance
        let config = MovementConfig {
            speed: 100.0,           // Very high speed
            stopping_distance: 0.1, // Default stopping distance
            ..MovementConfig::default()
        };

        let result = calculate_movement(current, Some(target), config);

        assert!(result.should_move);
        // Movement should be clamped to not overshoot
        assert!(result.movement_vector.length() <= result.distance_to_target);
        // Distance should be 0.5, movement should be less due to clamping
        assert_eq!(result.distance_to_target, 0.5);
        assert!(result.movement_vector.length() < config.speed * config.delta_time);
    }

    #[test]
    fn test_rotation_target_calculation() {
        let current = Vec3::new(0.0, 1.0, 0.0);
        let target = Vec3::new(1.0, 1.0, 0.0);
        let config = MovementConfig::default();

        let result = calculate_movement(current, Some(target), config);

        let rotation_target = result
            .rotation_target
            .expect("Rotation target should be calculated for valid movement");

        // For GLB models facing backwards, rotation target should be flipped
        assert!(rotation_target.x < current.x); // Flipped X direction
        assert_eq!(rotation_target.y, current.y); // Same Y level
    }

    #[test]
    fn test_target_validation() {
        let current = Vec3::new(0.0, 1.0, 0.0);

        // Valid target
        assert!(validate_target(current, Vec3::new(5.0, 1.0, 3.0)));

        // Too close
        assert!(!validate_target(current, Vec3::new(0.005, 1.0, 0.0)));

        // Too far
        assert!(!validate_target(current, Vec3::new(2000.0, 1.0, 0.0)));

        // Invalid coordinates
        assert!(!validate_target(current, Vec3::new(f32::NAN, 1.0, 0.0)));
        assert!(!validate_target(
            current,
            Vec3::new(5.0, f32::INFINITY, 0.0)
        ));
    }

    #[test]
    fn test_ray_to_ground_target() {
        let ray_origin = Vec3::new(0.0, 10.0, 0.0);
        let ray_direction = Vec3::new(1.0, -1.0, 1.0).normalize();
        let ground_y = 0.0;

        let result = ray_to_ground_target(ray_origin, ray_direction, ground_y);

        let target = result.expect("Ray should hit ground when pointing downward");
        assert_eq!(target.y, ground_y);
        assert!(target.x > 0.0);
        assert!(target.z > 0.0);
    }

    #[test]
    fn test_ray_parallel_to_ground() {
        let ray_origin = Vec3::new(0.0, 10.0, 0.0);
        let ray_direction = Vec3::new(1.0, 0.0, 1.0).normalize(); // No Y component
        let ground_y = 0.0;

        let result = ray_to_ground_target(ray_origin, ray_direction, ground_y);

        assert!(result.is_none());
    }

    #[test]
    fn test_ray_pointing_away_from_ground() {
        let ray_origin = Vec3::new(0.0, 10.0, 0.0);
        let ray_direction = Vec3::new(1.0, 1.0, 1.0).normalize(); // Pointing up
        let ground_y = 0.0;

        let result = ray_to_ground_target(ray_origin, ray_direction, ground_y);

        assert!(result.is_none());
    }
}
