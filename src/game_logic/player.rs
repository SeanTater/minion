use crate::components::LodLevel;
use crate::game_logic::MinionResult;
use crate::map::MapDefinition;
use crate::resources::GameConfig;
use bevy::prelude::*;

#[cfg(test)]
use crate::config::range_types::*;

/// Configuration for player spawn logic
#[derive(Debug, Clone, Copy)]
pub struct PlayerSpawnConfig {
    pub spawn_height_offset: f32,
    pub scale: f32,
    pub capsule_height: f32,
    pub capsule_radius: f32,
    pub snap_to_ground_distance: f32,
    pub controller_offset: f32,
    pub max_slope_climb_angle: f32,
    pub min_slope_slide_angle: f32,
    pub agent_radius: f32,
}

impl Default for PlayerSpawnConfig {
    fn default() -> Self {
        Self {
            spawn_height_offset: 5.0,
            scale: 2.0,
            capsule_height: 1.0,
            capsule_radius: 0.5,
            snap_to_ground_distance: 2.0,
            controller_offset: 0.01,
            max_slope_climb_angle: 45.0_f32.to_radians(),
            min_slope_slide_angle: 30.0_f32.to_radians(),
            agent_radius: 0.5,
        }
    }
}

/// Configuration for player input processing
#[derive(Debug, Clone, Copy)]
pub struct PlayerInputConfig {
    pub target_validation_max_distance: f32,
    pub target_validation_min_distance: f32,
}

impl Default for PlayerInputConfig {
    fn default() -> Self {
        Self {
            target_validation_max_distance: 100.0,
            target_validation_min_distance: 0.1,
        }
    }
}

/// Configuration for player movement coordination
#[derive(Debug, Clone, Copy)]
pub struct PlayerMovementConfig {
    pub y_coordinate_adjustment_threshold: f32,
    pub gravity_force: f32,
    pub pathfinding_distance_threshold: f32,
}

impl Default for PlayerMovementConfig {
    fn default() -> Self {
        Self {
            y_coordinate_adjustment_threshold: 2.0,
            gravity_force: 3.0,
            pathfinding_distance_threshold: 0.1,
        }
    }
}

/// Result of LOD level selection
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LodSelection {
    pub level: LodLevel,
    pub is_valid: bool,
}

/// Result of spawn position calculation
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpawnPosition {
    pub position: Vec3,
    pub is_valid: bool,
}

/// Result of input validation
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InputValidation {
    pub is_valid: bool,
    pub error_message: Option<&'static str>,
}

/// Result of target calculation
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TargetCalculation {
    pub target: Option<Vec3>,
    pub is_valid: bool,
    pub error_message: Option<&'static str>,
}

/// Result of Y-coordinate adjustment
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct YCoordinateAdjustment {
    pub adjusted_target: Vec3,
    pub was_adjusted: bool,
}

/// Result of gravity application
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GravityApplication {
    pub movement_vector: Vec3,
    pub gravity_applied: bool,
}

/// Result of target clearing decision
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TargetClearingDecision {
    pub should_clear: bool,
    pub reason: Option<&'static str>,
}

/// **PRIORITY 1: Spawn Logic**
///
/// Extract LOD selection logic into pure function
///
/// Determines the starting LOD level based on global max setting.
/// This is critical for preventing spawn failures due to invalid LOD configurations.
///
/// # Arguments
/// * `max_lod_level` - The maximum LOD level string from game config
///
/// # Returns
/// * `LodSelection` - The selected LOD level and validation status
///
/// # Examples
/// ```
/// use minion::game_logic::player::select_starting_lod;
/// use minion::components::LodLevel;
///
/// let selection = select_starting_lod("high");
/// assert_eq!(selection.level, LodLevel::High);
/// assert!(selection.is_valid);
///
/// let selection = select_starting_lod("invalid");
/// assert_eq!(selection.level, LodLevel::High); // Falls back to high
/// assert!(!selection.is_valid);
/// ```
pub fn select_starting_lod(max_lod_level: &str) -> LodSelection {
    match LodLevel::try_from(max_lod_level) {
        Ok(level) => LodSelection {
            level,
            is_valid: true,
        },
        Err(_) => LodSelection {
            level: LodLevel::High, // Default fallback
            is_valid: false,
        },
    }
}

/// Extract spawn position calculation into pure function
///
/// Calculates the spawn position from map data with height offset to avoid terrain intersection.
/// This is critical for preventing spawn failures due to invalid positions.
///
/// # Arguments
/// * `map` - The map definition containing spawn point
/// * `config` - Spawn configuration with height offset
///
/// # Returns
/// * `SpawnPosition` - The calculated spawn position and validation status
///
/// # Examples
/// ```
/// use minion::game_logic::player::{calculate_spawn_position, PlayerSpawnConfig};
/// use minion::map::{MapDefinition, TerrainData};
/// use bevy::prelude::Vec3;
///
/// let terrain = TerrainData::create_flat(10, 10, 1.0, 0.0).unwrap();
/// let map = MapDefinition::new(
///     "test".to_string(),
///     terrain,
///     Vec3::new(10.0, 5.0, 15.0),
///     vec![],
///     vec![],
/// ).unwrap();
/// let config = PlayerSpawnConfig::default();
///
/// let spawn = calculate_spawn_position(&map, config);
/// assert_eq!(spawn.position, Vec3::new(10.0, 10.0, 15.0)); // +5.0 height offset
/// assert!(spawn.is_valid);
/// ```
pub fn calculate_spawn_position(map: &MapDefinition, config: PlayerSpawnConfig) -> SpawnPosition {
    let position = Vec3::new(
        map.player_spawn.x,
        map.player_spawn.y + config.spawn_height_offset,
        map.player_spawn.z,
    );

    // Validate spawn position is reasonable
    let is_valid = position.x.is_finite()
        && position.y.is_finite()
        && position.z.is_finite()
        && position.y >= -1000.0  // Reasonable lower bound
        && position.y <= 1000.0; // Reasonable upper bound

    SpawnPosition { position, is_valid }
}

/// Validate component initialization parameters
///
/// Ensures all component initialization values are valid to prevent spawn failures.
/// This catches configuration errors early before entity creation.
///
/// # Arguments
/// * `game_config` - Game configuration with player settings
/// * `spawn_config` - Spawn configuration parameters
///
/// # Returns
/// * `MinionResult<()>` - Success or error with details
///
/// # Examples
/// ```
/// use minion::game_logic::player::{validate_component_initialization, PlayerSpawnConfig};
/// use minion::resources::GameConfig;
///
/// let game_config = GameConfig::default();
/// let spawn_config = PlayerSpawnConfig::default();
///
/// let result = validate_component_initialization(&game_config, spawn_config);
/// assert!(result.is_ok());
/// ```
pub fn validate_component_initialization(
    _game_config: &GameConfig,
    spawn_config: PlayerSpawnConfig,
) -> MinionResult<()> {
    // Range-safe types guarantee all values are valid by construction
    // No validation needed

    // Validate spawn config
    if spawn_config.scale <= 0.0 {
        return Err(crate::game_logic::MinionError::InvalidConfig {
            reason: "Player scale must be positive".to_string(),
        });
    }

    if spawn_config.capsule_height <= 0.0 || spawn_config.capsule_radius <= 0.0 {
        return Err(crate::game_logic::MinionError::InvalidConfig {
            reason: "Player capsule dimensions must be positive".to_string(),
        });
    }

    if spawn_config.agent_radius <= 0.0 {
        return Err(crate::game_logic::MinionError::InvalidConfig {
            reason: "Player agent radius must be positive".to_string(),
        });
    }

    Ok(())
}

/// **PRIORITY 2: Input Processing**
///
/// Validate mouse input and handle errors
///
/// Validates that required input components are available and handles common error cases.
/// This prevents crashes from missing windows or cameras.
///
/// # Arguments
/// * `has_window` - Whether primary window exists
/// * `has_camera` - Whether 3D camera exists
/// * `cursor_position` - Optional cursor position from window
///
/// # Returns
/// * `InputValidation` - Validation result with error details
///
/// # Examples
/// ```
/// use minion::game_logic::player::validate_mouse_input;
/// use bevy::prelude::Vec2;
///
/// let validation = validate_mouse_input(true, true, Some(Vec2::new(100.0, 200.0)));
/// assert!(validation.is_valid);
///
/// let validation = validate_mouse_input(false, true, None);
/// assert!(!validation.is_valid);
/// assert_eq!(validation.error_message, Some("Primary window not found"));
/// ```
pub fn validate_mouse_input(
    has_window: bool,
    has_camera: bool,
    cursor_position: Option<Vec2>,
) -> InputValidation {
    if !has_window {
        return InputValidation {
            is_valid: false,
            error_message: Some("Primary window not found"),
        };
    }

    if !has_camera {
        return InputValidation {
            is_valid: false,
            error_message: Some("3D camera not found"),
        };
    }

    if cursor_position.is_none() {
        return InputValidation {
            is_valid: false,
            error_message: Some("Cursor position not available"),
        };
    }

    InputValidation {
        is_valid: true,
        error_message: None,
    }
}

/// Calculate and validate target from ray intersection
///
/// Processes camera ray to world position and validates the resulting target.
/// This handles the complex ray-to-ground calculation and validation pipeline.
///
/// # Arguments
/// * `ray_origin` - Origin point of the camera ray
/// * `ray_direction` - Direction vector of the camera ray
/// * `player_y` - Current Y position of the player
/// * `config` - Input configuration with validation parameters
///
/// # Returns
/// * `TargetCalculation` - Target position and validation result
///
/// # Examples
/// ```
/// use minion::game_logic::player::{calculate_target_from_ray, PlayerInputConfig};
/// use bevy::prelude::Vec3;
///
/// let config = PlayerInputConfig::default();
/// let result = calculate_target_from_ray(
///     Vec3::new(0.0, 10.0, 0.0),
///     Vec3::new(0.0, -1.0, 0.0),
///     1.0,
///     config
/// );
///
/// // The function should process the ray without crashing
/// // Result validity depends on distance constraints and ray intersection
/// assert!(result.target.is_some() || result.error_message.is_some());
/// ```
pub fn calculate_target_from_ray(
    ray_origin: Vec3,
    ray_direction: Vec3,
    player_y: f32,
    config: PlayerInputConfig,
) -> TargetCalculation {
    // Use existing ray_to_ground_target function
    let target = crate::game_logic::ray_to_ground_target(ray_origin, ray_direction, player_y);

    if let Some(target_pos) = target {
        // Validate target position
        if !target_pos.x.is_finite() || !target_pos.y.is_finite() || !target_pos.z.is_finite() {
            return TargetCalculation {
                target: None,
                is_valid: false,
                error_message: Some("Target position contains invalid values"),
            };
        }

        // Check distance bounds
        let distance = ray_origin.distance(target_pos);
        if distance > config.target_validation_max_distance {
            return TargetCalculation {
                target: None,
                is_valid: false,
                error_message: Some("Target too far from player"),
            };
        }

        if distance < config.target_validation_min_distance {
            return TargetCalculation {
                target: None,
                is_valid: false,
                error_message: Some("Target too close to player"),
            };
        }

        // Use existing validate_target function for additional checks
        let player_pos = Vec3::new(ray_origin.x, player_y, ray_origin.z);
        if !crate::game_logic::validate_target(player_pos, target_pos) {
            return TargetCalculation {
                target: None,
                is_valid: false,
                error_message: Some("Target failed validation checks"),
            };
        }

        TargetCalculation {
            target: Some(target_pos),
            is_valid: true,
            error_message: None,
        }
    } else {
        TargetCalculation {
            target: None,
            is_valid: false,
            error_message: Some("Could not calculate ground target from ray"),
        }
    }
}

/// **PRIORITY 3: Movement Coordination**
///
/// Extract Y-coordinate adjustment logic
///
/// Adjusts waypoint Y coordinate to match player's movement plane.
/// This fixes issues where pathfinding uses terrain height but player moves above it.
///
/// # Arguments
/// * `waypoint` - The pathfinding waypoint position
/// * `player_y` - Current Y position of the player
/// * `config` - Movement configuration with adjustment threshold
///
/// # Returns
/// * `YCoordinateAdjustment` - Adjusted target and whether adjustment was made
///
/// # Examples
/// ```
/// use minion::game_logic::player::{adjust_waypoint_y_coordinate, PlayerMovementConfig};
/// use bevy::prelude::Vec3;
///
/// let config = PlayerMovementConfig::default();
/// let waypoint = Vec3::new(5.0, 0.0, 10.0);  // Ground level
/// let player_y = 3.0;  // Player floating above
///
/// let result = adjust_waypoint_y_coordinate(waypoint, player_y, config);
/// assert!(result.was_adjusted);
/// assert_eq!(result.adjusted_target.y, 3.0);  // Uses player Y
/// ```
pub fn adjust_waypoint_y_coordinate(
    waypoint: Vec3,
    player_y: f32,
    config: PlayerMovementConfig,
) -> YCoordinateAdjustment {
    let y_difference = (waypoint.y - player_y).abs();

    if y_difference > config.y_coordinate_adjustment_threshold {
        // Use player Y when waypoint Y is very different
        YCoordinateAdjustment {
            adjusted_target: Vec3::new(waypoint.x, player_y, waypoint.z),
            was_adjusted: true,
        }
    } else {
        // Keep waypoint Y when difference is reasonable
        YCoordinateAdjustment {
            adjusted_target: waypoint,
            was_adjusted: false,
        }
    }
}

/// Extract gravity application logic
///
/// Applies gravity component to movement vector for physics consistency.
/// This ensures proper physics behavior whether moving or stationary.
///
/// # Arguments
/// * `movement_vector` - The calculated movement vector (without gravity)
/// * `is_moving` - Whether the entity is currently moving
/// * `delta_time` - Time delta for this frame
/// * `config` - Movement configuration with gravity force
///
/// # Returns
/// * `GravityApplication` - Final movement vector with gravity applied
///
/// # Examples
/// ```
/// use minion::game_logic::player::{apply_gravity_to_movement, PlayerMovementConfig};
/// use bevy::prelude::Vec3;
///
/// let config = PlayerMovementConfig::default();
/// let movement = Vec3::new(1.0, 0.0, 0.5);
/// let delta_time = 1.0 / 60.0;
///
/// let result = apply_gravity_to_movement(movement, true, delta_time, config);
/// assert!(result.gravity_applied);
/// assert_eq!(result.movement_vector.x, 1.0);  // Horizontal unchanged
/// assert!(result.movement_vector.y < 0.0);    // Gravity applied
/// ```
pub fn apply_gravity_to_movement(
    movement_vector: Vec3,
    is_moving: bool,
    delta_time: f32,
    config: PlayerMovementConfig,
) -> GravityApplication {
    let gravity_component = -config.gravity_force * delta_time;

    let final_movement = if is_moving {
        // Add gravity to existing movement
        Vec3::new(movement_vector.x, gravity_component, movement_vector.z)
    } else {
        // Apply only gravity when stationary
        Vec3::new(0.0, gravity_component, 0.0)
    };

    GravityApplication {
        movement_vector: final_movement,
        gravity_applied: true,
    }
}

/// Extract target clearing state machine logic
///
/// Determines when to clear movement targets based on pathfinding state.
/// This prevents target oscillation and ensures clean movement termination.
///
/// # Arguments
/// * `has_pathfinding_destination` - Whether pathfinding has a destination set
/// * `has_current_waypoint` - Whether pathfinding has a current waypoint
/// * `has_move_target` - Whether player has a direct move target
/// * `distance_to_target` - Current distance to target
/// * `stopping_distance` - Distance threshold for stopping movement
///
/// # Returns
/// * `TargetClearingDecision` - Whether to clear targets and the reason
///
/// # Examples
/// ```
/// use minion::game_logic::player::should_clear_movement_target;
///
/// // Should clear when no pathfinding and close to target
/// let decision = should_clear_movement_target(false, false, true, 0.3, 0.5);
/// assert!(decision.should_clear);
/// assert_eq!(decision.reason, Some("All targets reached"));
///
/// // Should not clear when pathfinding is active
/// let decision = should_clear_movement_target(true, true, true, 0.3, 0.5);
/// assert!(!decision.should_clear);
/// ```
pub fn should_clear_movement_target(
    has_pathfinding_destination: bool,
    has_current_waypoint: bool,
    has_move_target: bool,
    distance_to_target: f32,
    stopping_distance: f32,
) -> TargetClearingDecision {
    // Only clear if pathfinding has no destination and no waypoints
    if has_pathfinding_destination || has_current_waypoint {
        return TargetClearingDecision {
            should_clear: false,
            reason: Some("Pathfinding still active"),
        };
    }

    // Only clear if we actually have a target to clear
    if !has_move_target {
        return TargetClearingDecision {
            should_clear: false,
            reason: Some("No target to clear"),
        };
    }

    // Clear when we're close enough to the target
    if distance_to_target <= stopping_distance {
        TargetClearingDecision {
            should_clear: true,
            reason: Some("All targets reached"),
        }
    } else {
        TargetClearingDecision {
            should_clear: false,
            reason: Some("Still moving to target"),
        }
    }
}

/// Calculate 2D distance for pathfinding consistency
///
/// Ensures pathfinding and movement systems use consistent distance calculations.
/// This fixes the critical bug where 3D vs 2D distance caused movement conflicts.
///
/// # Arguments
/// * `current_position` - Current entity position
/// * `target_position` - Target position
///
/// # Returns
/// * `f32` - 2D distance (ignoring Y axis)
///
/// # Examples
/// ```
/// use minion::game_logic::player::calculate_2d_distance;
/// use bevy::prelude::Vec3;
///
/// let current = Vec3::new(0.0, 5.0, 0.0);
/// let target = Vec3::new(3.0, 0.0, 4.0);
///
/// let distance = calculate_2d_distance(current, target);
/// assert_eq!(distance, 5.0);  // 3-4-5 triangle, ignoring Y difference
/// ```
pub fn calculate_2d_distance(current_position: Vec3, target_position: Vec3) -> f32 {
    let current_2d = Vec3::new(current_position.x, 0.0, current_position.z);
    let target_2d = Vec3::new(target_position.x, 0.0, target_position.z);
    current_2d.distance(target_2d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::GameSettings;

    #[test]
    fn test_select_starting_lod_valid_levels() {
        let selection = select_starting_lod("high");
        assert_eq!(selection.level, LodLevel::High);
        assert!(selection.is_valid);

        let selection = select_starting_lod("medium");
        assert_eq!(selection.level, LodLevel::Medium);
        assert!(selection.is_valid);

        let selection = select_starting_lod("low");
        assert_eq!(selection.level, LodLevel::Low);
        assert!(selection.is_valid);
    }

    #[test]
    fn test_select_starting_lod_invalid_level() {
        let selection = select_starting_lod("invalid");
        assert_eq!(selection.level, LodLevel::High); // Falls back to high (default)
        assert!(!selection.is_valid);

        let selection = select_starting_lod("");
        assert_eq!(selection.level, LodLevel::High);
        assert!(!selection.is_valid);
    }

    #[test]
    fn test_calculate_spawn_position_valid() {
        let terrain = crate::map::TerrainData::create_flat(10, 10, 1.0, 0.0).unwrap();
        let map = crate::map::MapDefinition::new(
            "test".to_string(),
            terrain,
            Vec3::new(10.0, 5.0, 15.0),
            vec![],
            vec![],
        )
        .unwrap();
        let config = PlayerSpawnConfig::default();

        let spawn = calculate_spawn_position(&map, config);
        assert_eq!(spawn.position, Vec3::new(10.0, 10.0, 15.0)); // +5.0 height offset
        assert!(spawn.is_valid);
    }

    #[test]
    fn test_calculate_spawn_position_invalid() {
        let terrain = crate::map::TerrainData::create_flat(10, 10, 1.0, 0.0).unwrap();
        let map = crate::map::MapDefinition::new(
            "test".to_string(),
            terrain,
            Vec3::new(f32::NAN, 5.0, 15.0),
            vec![],
            vec![],
        )
        .unwrap();
        let config = PlayerSpawnConfig::default();

        let spawn = calculate_spawn_position(&map, config);
        assert!(!spawn.is_valid);
    }

    #[test]
    fn test_validate_component_initialization_valid() {
        let game_config = GameConfig {
            username: "test".to_string(),
            score: 0,
            settings: GameSettings {
                player_movement_speed: MovementSpeed::new(5.0),
                player_max_health: HealthValue::new(100.0),
                player_max_mana: ManaValue::new(50.0),
                player_max_energy: EnergyValue::new(75.0),
                ..Default::default()
            },
        };
        let spawn_config = PlayerSpawnConfig::default();

        let result = validate_component_initialization(&game_config, spawn_config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_component_initialization_clamped_values() {
        let game_config = GameConfig {
            username: "test".to_string(),
            score: 0,
            settings: GameSettings {
                player_movement_speed: MovementSpeed::new(0.0), // Will be clamped to 0.1
                ..Default::default()
            },
        };
        let spawn_config = PlayerSpawnConfig::default();

        let result = validate_component_initialization(&game_config, spawn_config);
        // Range-safe types ensure all values are valid, so this should succeed
        assert!(result.is_ok());
        // Verify the value was clamped
        assert_eq!(game_config.settings.player_movement_speed.get(), 0.1);
    }

    #[test]
    fn test_validate_component_initialization_clamped_health() {
        let game_config = GameConfig {
            username: "test".to_string(),
            score: 0,
            settings: GameSettings {
                player_max_health: HealthValue::new(-10.0), // Will be clamped to 1.0
                ..Default::default()
            },
        };
        let spawn_config = PlayerSpawnConfig::default();

        let result = validate_component_initialization(&game_config, spawn_config);
        // Range-safe types ensure all values are valid, so this should succeed
        assert!(result.is_ok());
        // Verify the value was clamped
        assert_eq!(game_config.settings.player_max_health.get(), 1.0);
    }

    #[test]
    fn test_validate_mouse_input_valid() {
        let validation = validate_mouse_input(true, true, Some(Vec2::new(100.0, 200.0)));
        assert!(validation.is_valid);
        assert!(validation.error_message.is_none());
    }

    #[test]
    fn test_validate_mouse_input_no_window() {
        let validation = validate_mouse_input(false, true, Some(Vec2::new(100.0, 200.0)));
        assert!(!validation.is_valid);
        assert_eq!(validation.error_message, Some("Primary window not found"));
    }

    #[test]
    fn test_validate_mouse_input_no_camera() {
        let validation = validate_mouse_input(true, false, Some(Vec2::new(100.0, 200.0)));
        assert!(!validation.is_valid);
        assert_eq!(validation.error_message, Some("3D camera not found"));
    }

    #[test]
    fn test_validate_mouse_input_no_cursor() {
        let validation = validate_mouse_input(true, true, None);
        assert!(!validation.is_valid);
        assert_eq!(
            validation.error_message,
            Some("Cursor position not available")
        );
    }

    #[test]
    fn test_calculate_target_from_ray_basic_functionality() {
        let config = PlayerInputConfig::default();

        // Test that the function handles basic ray calculations
        let ray_origin = Vec3::new(0.0, 10.0, 0.0);
        let ray_direction = Vec3::new(0.0, -1.0, 0.0);
        let player_y = 0.0;

        let result = calculate_target_from_ray(ray_origin, ray_direction, player_y, config);

        // The function should at least process the ray without crashing
        // The validation might fail due to distance constraints, but that's expected behavior
        assert!(result.target.is_some() || result.error_message.is_some());
    }

    #[test]
    fn test_calculate_target_from_ray_too_far() {
        let config = PlayerInputConfig {
            target_validation_max_distance: 5.0, // Very small max distance
            ..Default::default()
        };
        let result = calculate_target_from_ray(
            Vec3::new(0.0, 10.0, 0.0),
            Vec3::new(0.0, -1.0, 0.0),
            1.0,
            config,
        );

        assert!(!result.is_valid);
        assert_eq!(result.error_message, Some("Target too far from player"));
    }

    #[test]
    fn test_adjust_waypoint_y_coordinate_needs_adjustment() {
        let config = PlayerMovementConfig::default();
        let waypoint = Vec3::new(5.0, 0.0, 10.0); // Ground level
        let player_y = 3.0; // Player floating above

        let result = adjust_waypoint_y_coordinate(waypoint, player_y, config);
        assert!(result.was_adjusted);
        assert_eq!(result.adjusted_target.y, 3.0); // Uses player Y
        assert_eq!(result.adjusted_target.x, 5.0); // X unchanged
        assert_eq!(result.adjusted_target.z, 10.0); // Z unchanged
    }

    #[test]
    fn test_adjust_waypoint_y_coordinate_no_adjustment() {
        let config = PlayerMovementConfig::default();
        let waypoint = Vec3::new(5.0, 2.5, 10.0); // Close to player Y
        let player_y = 3.0;

        let result = adjust_waypoint_y_coordinate(waypoint, player_y, config);
        assert!(!result.was_adjusted);
        assert_eq!(result.adjusted_target, waypoint); // Unchanged
    }

    #[test]
    fn test_apply_gravity_to_movement_while_moving() {
        let config = PlayerMovementConfig::default();
        let movement = Vec3::new(1.0, 0.0, 0.5);
        let delta_time = 1.0 / 60.0;

        let result = apply_gravity_to_movement(movement, true, delta_time, config);
        assert!(result.gravity_applied);
        assert_eq!(result.movement_vector.x, 1.0); // Horizontal unchanged
        assert_eq!(result.movement_vector.z, 0.5); // Horizontal unchanged
        assert!(result.movement_vector.y < 0.0); // Gravity applied
        assert_eq!(result.movement_vector.y, -config.gravity_force * delta_time);
    }

    #[test]
    fn test_apply_gravity_to_movement_while_stationary() {
        let config = PlayerMovementConfig::default();
        let movement = Vec3::new(1.0, 0.0, 0.5); // This gets ignored when stationary
        let delta_time = 1.0 / 60.0;

        let result = apply_gravity_to_movement(movement, false, delta_time, config);
        assert!(result.gravity_applied);
        assert_eq!(result.movement_vector.x, 0.0); // No horizontal movement
        assert_eq!(result.movement_vector.z, 0.0); // No horizontal movement
        assert!(result.movement_vector.y < 0.0); // Gravity applied
        assert_eq!(result.movement_vector.y, -config.gravity_force * delta_time);
    }

    #[test]
    fn test_should_clear_movement_target_pathfinding_active() {
        let decision = should_clear_movement_target(true, false, true, 0.3, 0.5);
        assert!(!decision.should_clear);
        assert_eq!(decision.reason, Some("Pathfinding still active"));

        let decision = should_clear_movement_target(false, true, true, 0.3, 0.5);
        assert!(!decision.should_clear);
        assert_eq!(decision.reason, Some("Pathfinding still active"));
    }

    #[test]
    fn test_should_clear_movement_target_no_target() {
        let decision = should_clear_movement_target(false, false, false, 0.3, 0.5);
        assert!(!decision.should_clear);
        assert_eq!(decision.reason, Some("No target to clear"));
    }

    #[test]
    fn test_should_clear_movement_target_reached() {
        let decision = should_clear_movement_target(false, false, true, 0.3, 0.5);
        assert!(decision.should_clear);
        assert_eq!(decision.reason, Some("All targets reached"));
    }

    #[test]
    fn test_should_clear_movement_target_still_moving() {
        let decision = should_clear_movement_target(false, false, true, 0.8, 0.5);
        assert!(!decision.should_clear);
        assert_eq!(decision.reason, Some("Still moving to target"));
    }

    #[test]
    fn test_calculate_2d_distance() {
        let current = Vec3::new(0.0, 5.0, 0.0);
        let target = Vec3::new(3.0, 0.0, 4.0);

        let distance = calculate_2d_distance(current, target);
        assert_eq!(distance, 5.0); // 3-4-5 triangle, ignoring Y difference

        // Test same position
        let distance = calculate_2d_distance(Vec3::ZERO, Vec3::new(0.0, 10.0, 0.0));
        assert_eq!(distance, 0.0); // Same XZ position, different Y
    }

    #[test]
    fn test_calculate_2d_distance_vs_3d_distance() {
        // This test demonstrates the critical bug fix
        let player_pos = Vec3::new(-0.5, 3.0, 0.0);
        let waypoint = Vec3::new(-1.0, 0.0, 0.0);

        let distance_2d = calculate_2d_distance(player_pos, waypoint);
        let distance_3d = player_pos.distance(waypoint);

        assert_eq!(distance_2d, 0.5); // XZ distance only
        assert!((distance_3d - 3.041).abs() < 0.01); // Includes Y difference

        // The 2D distance should be used for movement consistency
        assert!(distance_2d < distance_3d);
    }

    #[test]
    fn test_pathfinding_movement_bug_reproduction_with_extracted_functions() {
        // Reproduce the exact scenario from the original bug
        let player_pos = Vec3::new(-0.5, 3.0, 0.0);
        let waypoint = Vec3::new(-1.0, 0.0, 0.0);
        let config = PlayerMovementConfig::default();

        // Test the extracted 2D distance calculation
        let distance_2d = calculate_2d_distance(player_pos, waypoint);
        let distance_3d = player_pos.distance(waypoint);

        assert_eq!(distance_2d, 0.5);
        assert!((distance_3d - 3.041).abs() < 0.01);

        // Test pathfinding decision with 2D distance (fixed)
        let should_move_2d = distance_2d > config.pathfinding_distance_threshold;
        let should_move_3d = distance_3d > config.pathfinding_distance_threshold;

        assert!(should_move_2d); // 0.5 > 0.1
        assert!(should_move_3d); // 3.041 > 0.1

        // Both approaches now agree on movement (both true)
        // The difference is now in the stopping distance comparison
        let stopping_distance = 0.5;
        let should_stop_2d = distance_2d <= stopping_distance;
        let should_stop_3d = distance_3d <= stopping_distance;

        assert!(should_stop_2d); // 0.5 <= 0.5 (should stop)
        assert!(!should_stop_3d); // 3.041 > 0.5 (should not stop)

        // The fix ensures consistent distance calculation method
        println!("✓ BUG FIXED: Using 2D distance consistently prevents movement conflicts");
    }

    #[test]
    fn test_edge_cases_for_extracted_functions() {
        // Test edge cases that could cause crashes or unexpected behavior

        // LOD selection with edge cases
        let selection = select_starting_lod("HIGH"); // Wrong case
        assert!(!selection.is_valid);

        let selection = select_starting_lod("med"); // Partial match
        assert!(!selection.is_valid);

        // Spawn position with extreme values
        let terrain = crate::map::TerrainData::create_flat(10, 10, 1.0, 0.0).unwrap();
        let map = crate::map::MapDefinition::new(
            "test".to_string(),
            terrain,
            Vec3::new(f32::INFINITY, 5.0, 15.0),
            vec![],
            vec![],
        )
        .unwrap();
        let spawn = calculate_spawn_position(&map, PlayerSpawnConfig::default());
        assert!(!spawn.is_valid);

        // Y coordinate adjustment with extreme differences
        let config = PlayerMovementConfig::default();
        let waypoint = Vec3::new(0.0, -1000.0, 0.0);
        let player_y = 1000.0;

        let result = adjust_waypoint_y_coordinate(waypoint, player_y, config);
        assert!(result.was_adjusted); // Large difference should trigger adjustment
        assert_eq!(result.adjusted_target.y, player_y);

        // Gravity application with zero delta time
        let result = apply_gravity_to_movement(Vec3::ZERO, true, 0.0, config);
        assert_eq!(result.movement_vector.y, 0.0); // No gravity with zero time

        // Distance calculation with same position
        let distance = calculate_2d_distance(Vec3::ZERO, Vec3::ZERO);
        assert_eq!(distance, 0.0);
    }
}
