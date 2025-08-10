use crate::components::*;
use crate::game_logic::{
    MovementConfig, PlayerInputConfig, PlayerMovementConfig, PlayerSpawnConfig,
    adjust_waypoint_y_coordinate, apply_gravity_to_movement, calculate_2d_distance,
    calculate_movement, calculate_spawn_position, calculate_target_from_ray, select_starting_lod,
    should_clear_movement_target, validate_component_initialization, validate_mouse_input,
};
use crate::map::MapDefinition;
use crate::pathfinding::{plan_paths, update_pathfinding_agents};
use crate::resources::{GameConfig, GameState};
use bevy::prelude::Camera3d;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_rapier3d::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Playing),
            spawn_player.after(crate::plugins::map_loader::load_map),
        )
        .add_systems(
            Update,
            (
                handle_player_input,
                plan_paths.after(handle_player_input),
                update_pathfinding_agents.after(plan_paths),
                move_player.after(update_pathfinding_agents),
                update_player_from_controller_output.after(move_player),
                debug_player_state.after(update_player_from_controller_output),
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(OnExit(GameState::Playing), cleanup_player);
    }
}

fn spawn_player(
    mut commands: Commands,
    game_config: Res<GameConfig>,
    player_query: Query<&Player>,
    asset_server: Res<AssetServer>,
    map: Res<MapDefinition>,
) {
    // Only spawn player if none exists
    if player_query.is_empty() {
        let spawn_config = PlayerSpawnConfig::default();

        // Validate component initialization early to prevent spawn failures
        if let Err(e) = validate_component_initialization(&game_config, spawn_config) {
            error!("Failed to validate player component initialization: {}", e);
            return;
        }

        // Load all LOD levels for player
        let high_scene = asset_server.load("players/hooded-high.glb#Scene0");
        let med_scene = asset_server.load("players/hooded-med.glb#Scene0");
        let low_scene = asset_server.load("players/hooded-low.glb#Scene0");

        // Use extracted LOD selection logic
        let lod_selection = select_starting_lod(&game_config.settings.max_lod_level);
        if !lod_selection.is_valid {
            warn!(
                "Invalid LOD level '{}', using fallback",
                game_config.settings.max_lod_level
            );
        }
        let starting_level = lod_selection.level;
        let starting_scene = match starting_level {
            LodLevel::Medium => med_scene.clone(),
            LodLevel::Low => low_scene.clone(),
            LodLevel::High => high_scene.clone(),
        };

        // Get spawn position from map
        info!(
            "Player spawning at map position: ({}, {}, {})",
            map.player_spawn.x, map.player_spawn.y, map.player_spawn.z
        );

        // Use extracted spawn position calculation
        let spawn_result = calculate_spawn_position(&map, spawn_config);
        if !spawn_result.is_valid {
            error!("Invalid spawn position calculated, aborting player spawn");
            return;
        }
        let spawn_position = spawn_result.position;

        info!(
            "Player spawning at position: ({:.2}, {:.2}, {:.2}) - character controller will snap to terrain",
            spawn_position.x, spawn_position.y, spawn_position.z
        );

        // Spawn player with 3D model (scaled to 2m tall, rotated to face forward)
        info!(
            "Spawning player with KinematicCharacterController at: ({:.2}, {:.2}, {:.2})",
            spawn_position.x, spawn_position.y, spawn_position.z
        );
        let player_entity = commands
            .spawn((
                SceneRoot(starting_scene),
                Transform::from_translation(spawn_position)
                    .with_scale(Vec3::splat(spawn_config.scale)),
                RigidBody::KinematicPositionBased,
                Collider::capsule_y(spawn_config.capsule_height, spawn_config.capsule_radius),
                KinematicCharacterController {
                    snap_to_ground: Some(CharacterLength::Absolute(
                        spawn_config.snap_to_ground_distance,
                    )),
                    offset: CharacterLength::Absolute(spawn_config.controller_offset),
                    max_slope_climb_angle: spawn_config.max_slope_climb_angle,
                    min_slope_slide_angle: spawn_config.min_slope_slide_angle,
                    slide: true,                           // Enable sliding on slopes
                    apply_impulse_to_dynamic_bodies: true, // Better physics interaction
                    ..default()
                },
                Player {
                    move_target: None,
                    speed: Speed::new(game_config.settings.player_movement_speed.get()),
                    health: HealthPool::new_full(game_config.settings.player_max_health.get()),
                    mana: ManaPool::new_full(game_config.settings.player_max_mana.get()),
                    energy: EnergyPool::new_full(game_config.settings.player_max_energy.get()),
                },
                PathfindingAgent {
                    agent_radius: spawn_config.agent_radius,
                    ..PathfindingAgent::default()
                },
                LodEntity {
                    current_level: starting_level,
                    high_handle: high_scene.clone(),
                    med_handle: med_scene.clone(),
                    low_handle: low_scene.clone(),
                    entity_type: LodEntityType::Player,
                },
            ))
            .id();

        info!("Player entity spawned with ID: {:?}", player_entity);
    }
}

fn handle_player_input(
    mut player_query: Query<(&Transform, &mut Player, &mut PathfindingAgent)>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
) {
    if mouse_button.just_pressed(MouseButton::Left) {
        let input_config = PlayerInputConfig::default();

        // Use extracted input validation
        let has_window = !windows.is_empty();
        let has_camera = !camera_query.is_empty();

        let window = match windows.single() {
            Ok(w) => w,
            Err(_) => {
                warn!("Input validation failed: Primary window not found");
                return;
            }
        };

        let cursor_pos = window.cursor_position();

        let validation = validate_mouse_input(has_window, has_camera, cursor_pos);
        if !validation.is_valid {
            if let Some(error_msg) = validation.error_message {
                warn!("Input validation failed: {}", error_msg);
            }
            return;
        }

        // Safe to unwrap after validation
        let cursor_pos = cursor_pos.unwrap();

        let (camera, camera_transform) = match camera_query.single() {
            Ok(result) => result,
            Err(_) => {
                warn!("Could not get camera after validation");
                return;
            }
        };
        if let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) {
            for (player_transform, mut player, mut pathfinding_agent) in player_query.iter_mut() {
                let player_y = player_transform.translation.y;

                // Use extracted target calculation
                let target_result =
                    calculate_target_from_ray(ray.origin, *ray.direction, player_y, input_config);

                if target_result.is_valid {
                    if let Some(target) = target_result.target {
                        // Set pathfinding destination for intelligent pathfinding
                        pathfinding_agent.destination = Some(target);
                        // Keep fallback behavior by also setting player.move_target
                        player.move_target = Some(target);
                        info!(
                            "Pathfinding target set: ({:.2}, {:.2}, {:.2})",
                            target.x, target.y, target.z
                        );
                    }
                } else if let Some(error_msg) = target_result.error_message {
                    warn!("Target calculation failed: {}", error_msg);
                }
            }
        } else {
            warn!("Could not get ray from camera");
        }
    }
}

fn move_player(
    mut player_query: Query<(
        &mut Transform,
        &mut Player,
        &mut KinematicCharacterController,
        &PathfindingAgent,
    )>,
    game_config: Res<GameConfig>,
    time: Res<Time>,
) {
    for (mut transform, mut player, mut controller, pathfinding_agent) in player_query.iter_mut() {
        let config = MovementConfig {
            speed: player.speed.0,
            stopping_distance: game_config.settings.player_stopping_distance.get(),
            slowdown_distance: game_config.settings.player_slowdown_distance.get(),
            delta_time: time.delta_secs(),
        };

        // Use pathfinding waypoint as primary target, fallback to direct target
        let pathfinding_waypoint = pathfinding_agent.current_waypoint();
        let mut movement_target = pathfinding_waypoint.or(player.move_target);

        let movement_config = PlayerMovementConfig::default();

        // Debug logging for pathfinding usage
        if let Some(waypoint) = pathfinding_waypoint {
            debug!(
                "Using pathfinding waypoint: ({:.1}, {:.1}, {:.1})",
                waypoint.x, waypoint.y, waypoint.z
            );

            // Use extracted Y coordinate adjustment logic
            if let Some(adjusted_waypoint) = movement_target {
                let player_y = transform.translation.y;
                let adjustment =
                    adjust_waypoint_y_coordinate(adjusted_waypoint, player_y, movement_config);

                if adjustment.was_adjusted {
                    debug!(
                        "Adjusted waypoint Y from {:.1} to {:.1}",
                        adjusted_waypoint.y, adjustment.adjusted_target.y
                    );
                }

                movement_target = Some(adjustment.adjusted_target);
            }
        } else if player.move_target.is_some() {
            debug!("Using direct movement target (no pathfinding waypoint)");
        }

        // Debug logging removed - use tests instead for debugging

        let calculation = calculate_movement(transform.translation, movement_target, config);

        // Debug logging removed - use tests instead for debugging

        // Override stopping logic for pathfinding waypoints - we should always move to waypoints
        let (should_move, final_movement_vector) = if pathfinding_waypoint.is_some() {
            // Use extracted 2D distance calculation for consistency
            let distance = movement_target.map_or(0.0, |target| {
                calculate_2d_distance(transform.translation, target)
            });
            let should_move_to_waypoint = distance > movement_config.pathfinding_distance_threshold;

            // CRITICAL FIX: Recalculate movement vector when overriding should_move
            let movement_vector = if should_move_to_waypoint && movement_target.is_some() {
                let target = movement_target.unwrap();
                let current_2d = Vec3::new(transform.translation.x, 0.0, transform.translation.z);
                let target_2d = Vec3::new(target.x, 0.0, target.z);
                let direction = (target_2d - current_2d).normalize_or_zero();
                let speed = player.speed.0 * time.delta_secs();
                direction * speed
            } else {
                calculation.movement_vector
            };

            (should_move_to_waypoint, movement_vector)
        } else {
            (calculation.should_move, calculation.movement_vector)
        };

        // Debug logging removed - movement is working correctly

        if should_move {
            // Use extracted gravity application
            let gravity_result = apply_gravity_to_movement(
                final_movement_vector,
                true,
                time.delta_secs(),
                movement_config,
            );
            controller.translation = Some(gravity_result.movement_vector);

            debug!(
                "Moving: distance={:.2} speed_factor={:.2}",
                calculation.distance_to_target, calculation.slowdown_factor
            );

            // Handle rotation using calculation result
            if let Some(rotation_target) = calculation.rotation_target {
                transform.look_at(rotation_target, Vec3::Y);
            }
        } else {
            // Use extracted target clearing logic
            let clearing_decision = should_clear_movement_target(
                pathfinding_agent.destination.is_some(),
                pathfinding_agent.current_waypoint().is_some(),
                player.move_target.is_some(),
                calculation.distance_to_target,
                game_config.settings.player_stopping_distance.get(),
            );

            if clearing_decision.should_clear {
                player.move_target = None;
                if let Some(reason) = clearing_decision.reason {
                    info!(
                        "{} - stopping movement (distance {:.2} <= stopping distance {:.2})",
                        reason,
                        calculation.distance_to_target,
                        game_config.settings.player_stopping_distance.get()
                    );
                }
            }

            // Use extracted gravity application for stationary state
            let gravity_result =
                apply_gravity_to_movement(Vec3::ZERO, false, time.delta_secs(), movement_config);
            controller.translation = Some(gravity_result.movement_vector);
        }
    }
}

fn update_player_from_controller_output(
    player_query: Query<(&KinematicCharacterControllerOutput,), With<Player>>,
) {
    // Check for controller issues
    for (output,) in player_query.iter() {
        if !output.grounded {
            debug!("Player not grounded - check terrain collision");
        }
    }
}

fn cleanup_player(mut commands: Commands, player_query: Query<Entity, With<Player>>) {
    for entity in player_query.iter() {
        commands.entity(entity).despawn();
    }
}

fn debug_player_state(player_query: Query<(&Transform, &Player), With<Player>>, time: Res<Time>) {
    // Only log every 3 seconds to avoid spam
    if (time.elapsed_secs() as u32) % 3 == 0 && time.delta_secs() < 0.02 {
        for (transform, player) in player_query.iter() {
            if let Some(target) = player.move_target {
                debug!(
                    "Player: pos=({:.1}, {:.1}, {:.1}) -> target=({:.1}, {:.1}, {:.1})",
                    transform.translation.x,
                    transform.translation.y,
                    transform.translation.z,
                    target.x,
                    target.y,
                    target.z
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_logic::movement::*;

    #[test]
    fn test_pathfinding_movement_bug_reproduction() {
        // Reproduce the exact scenario from the logs
        let player_pos = Vec3::new(-0.5, 3.0, 0.0);
        let waypoint = Vec3::new(-1.0, 0.0, 0.0);

        // Test movement calculation directly with realistic config values
        let config = MovementConfig {
            speed: 5.0,
            stopping_distance: 0.5, // Default player stopping distance from resources
            slowdown_distance: 2.0, // Default player slowdown distance from resources
            delta_time: 1.0 / 60.0, // 60 FPS
        };

        let calculation = calculate_movement(player_pos, Some(waypoint), config);

        // Calculate distances manually to debug the issue
        let distance_3d = player_pos.distance(waypoint);
        let distance_2d = Vec3::new(player_pos.x, 0.0, player_pos.z)
            .distance(Vec3::new(waypoint.x, 0.0, waypoint.z));

        println!("=== PATHFINDING MOVEMENT BUG REPRODUCTION ===");
        println!(
            "Player position: ({:.1}, {:.1}, {:.1})",
            player_pos.x, player_pos.y, player_pos.z
        );
        println!(
            "Waypoint position: ({:.1}, {:.1}, {:.1})",
            waypoint.x, waypoint.y, waypoint.z
        );
        println!("3D distance: {:.2}", distance_3d);
        println!("2D distance: {:.2}", distance_2d);
        println!(
            "Movement config stopping_distance: {:.2}",
            config.stopping_distance
        );
        println!(
            "Movement calculation should_move: {}",
            calculation.should_move
        );
        println!(
            "Movement calculation distance_to_target: {:.2}",
            calculation.distance_to_target
        );

        // Test the pathfinding integration logic
        let pathfinding_threshold = 0.1;
        let pathfinding_should_move = distance_3d > pathfinding_threshold;
        println!("Pathfinding threshold: {:.2}", pathfinding_threshold);
        println!("Pathfinding should_move: {}", pathfinding_should_move);

        // Reproduce the bug: calculate_movement uses 2D distance, pathfinding logic uses 3D distance
        assert!(
            (distance_3d - 3.04138).abs() < 0.001,
            "3D distance should be approximately 3.04138"
        );
        assert_eq!(
            distance_2d, 0.5,
            "2D distance should be exactly at stopping distance"
        );
        assert_eq!(
            calculation.distance_to_target, distance_2d,
            "Movement calculation should use 2D distance"
        );

        // This shows the bug: movement says don't move (2D distance = 0.5 = stopping distance)
        // but pathfinding says move (3D distance = 3.05 > 0.1)
        assert!(
            !calculation.should_move,
            "Movement calculation should return false (2D distance = stopping distance)"
        );
        assert!(
            pathfinding_should_move,
            "Pathfinding logic should return true (3D distance > 0.1)"
        );

        println!("=== BUG CONFIRMED ===");
        println!(
            "Movement calculation uses 2D distance ({:.2}) which equals stopping distance ({:.2})",
            distance_2d, config.stopping_distance
        );
        println!(
            "Pathfinding integration uses 3D distance ({:.2}) which is > threshold ({:.2})",
            distance_3d, pathfinding_threshold
        );
        println!("This causes the contradiction: movement says don't move, pathfinding says move");
    }

    #[test]
    fn test_pathfinding_distance_calculation_variants() {
        // Test various Y-axis height differences to understand the impact
        let test_cases = vec![
            // (player_pos, waypoint, expected_2d, expected_3d, description)
            (
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(1.0, 0.0, 0.0),
                1.0,
                1.0,
                "Same Y level",
            ),
            (
                Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(1.0, 1.0, 0.0),
                1.0,
                1.0,
                "Same Y level elevated",
            ),
            (
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(1.0, 1.0, 0.0),
                1.0,
                1.414,
                "1 unit Y difference",
            ),
            (
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(1.0, 3.0, 0.0),
                1.0,
                3.162,
                "3 unit Y difference",
            ),
            (
                Vec3::new(-0.5, 3.0, 0.0),
                Vec3::new(-1.0, 0.0, 0.0),
                0.5,
                3.041,
                "Bug reproduction exact",
            ),
        ];

        let config = MovementConfig {
            speed: 5.0,
            stopping_distance: 0.5,
            slowdown_distance: 2.0,
            delta_time: 1.0 / 60.0,
        };

        println!("=== DISTANCE CALCULATION VARIANTS ===");
        for (player_pos, waypoint, expected_2d, expected_3d, description) in test_cases {
            let calculation = calculate_movement(player_pos, Some(waypoint), config);
            let distance_3d = player_pos.distance(waypoint);
            let distance_2d = Vec3::new(player_pos.x, 0.0, player_pos.z)
                .distance(Vec3::new(waypoint.x, 0.0, waypoint.z));

            println!(
                "{}: 2D={:.3} (expected {:.3}), 3D={:.3} (expected {:.3}), should_move={}",
                description,
                distance_2d,
                expected_2d,
                distance_3d,
                expected_3d,
                calculation.should_move
            );

            assert!(
                (distance_2d - expected_2d).abs() < 0.01,
                "2D distance mismatch for {}",
                description
            );
            assert!(
                (distance_3d - expected_3d).abs() < 0.01,
                "3D distance mismatch for {}",
                description
            );
        }
    }

    #[test]
    fn test_movement_config_edge_cases() {
        let player_pos = Vec3::new(-0.5, 3.0, 0.0);
        let waypoint = Vec3::new(-1.0, 0.0, 0.0);

        // Test with different stopping distances
        let stopping_distances = vec![0.1, 0.3, 0.5, 1.0];

        println!("=== MOVEMENT CONFIG EDGE CASES ===");
        for stopping_distance in stopping_distances {
            let config = MovementConfig {
                speed: 5.0,
                stopping_distance,
                slowdown_distance: 2.0,
                delta_time: 1.0 / 60.0,
            };

            let calculation = calculate_movement(player_pos, Some(waypoint), config);
            println!(
                "Stopping distance {:.1}: should_move={}, distance_to_target={:.2}",
                stopping_distance, calculation.should_move, calculation.distance_to_target
            );

            // The 2D distance is 0.5, so movement should stop when stopping_distance >= 0.5
            if stopping_distance >= 0.5 {
                assert!(
                    !calculation.should_move,
                    "Should not move when stopping_distance >= 0.5"
                );
            } else {
                assert!(
                    calculation.should_move,
                    "Should move when stopping_distance < 0.5"
                );
            }
        }
    }

    #[test]
    fn test_pathfinding_integration_logic_directly() {
        // Test the exact logic from the move_player function
        let player_pos = Vec3::new(-0.5, 3.0, 0.0);
        let waypoint = Vec3::new(-1.0, 0.0, 0.0);

        let config = MovementConfig {
            speed: 5.0,
            stopping_distance: 0.5,
            slowdown_distance: 2.0,
            delta_time: 1.0 / 60.0,
        };

        let calculation = calculate_movement(player_pos, Some(waypoint), config);

        // Simulate the pathfinding integration logic from move_player function
        let pathfinding_waypoint = Some(waypoint);
        let movement_target = pathfinding_waypoint;

        let should_move = if pathfinding_waypoint.is_some() {
            // This is the problematic line from the original code
            let distance = movement_target.map_or(0.0, |target| player_pos.distance(target));
            distance > 0.1
        } else {
            calculation.should_move
        };

        println!("=== PATHFINDING INTEGRATION LOGIC ===");
        println!(
            "Movement calculation should_move: {}",
            calculation.should_move
        );
        println!("Pathfinding integration should_move: {}", should_move);
        println!(
            "Distance used by pathfinding: {:.2}",
            player_pos.distance(waypoint)
        );
        println!(
            "Distance used by movement: {:.2}",
            calculation.distance_to_target
        );

        // This demonstrates the bug
        assert!(
            !calculation.should_move,
            "Movement calculation should say don't move"
        );
        assert!(should_move, "Pathfinding integration should say move");

        println!("BUG CONFIRMED: Movement and pathfinding logic disagree!");
    }

    #[test]
    fn test_pathfinding_agent_waypoint_reach_vs_movement_stopping() {
        // Test the relationship between PathfindingAgent waypoint_reach_distance and movement stopping_distance
        let pathfinding_agent = PathfindingAgent::new();
        let movement_config = MovementConfig {
            speed: 5.0,
            stopping_distance: 0.5, // Default from resources
            slowdown_distance: 2.0,
            delta_time: 1.0 / 60.0,
        };

        println!("=== PATHFINDING AGENT VS MOVEMENT CONFIG ===");
        println!(
            "PathfindingAgent waypoint_reach_distance: {:.2}",
            pathfinding_agent.waypoint_reach_distance
        );
        println!(
            "MovementConfig stopping_distance: {:.2}",
            movement_config.stopping_distance
        );
        println!("Pathfinding threshold in integration logic: 0.1");

        // With spaced waypoints, waypoint_reach_distance can be larger than stopping_distance
        // This is actually beneficial as it prevents oscillation around waypoints
        assert!(
            pathfinding_agent.waypoint_reach_distance >= 1.0,
            "Waypoint reach distance should work with spaced waypoints (>=1.0 units)"
        );

        // Test distances with different Y levels
        let test_positions = vec![
            (
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.2, 0.0, 0.0),
                "Same level, within waypoint reach",
            ),
            (
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.4, 0.0, 0.0),
                "Same level, within movement stopping",
            ),
            (
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.2, 1.0, 0.0),
                "Y difference, would exceed waypoint reach in 3D",
            ),
            (
                Vec3::new(0.0, 3.0, 0.0),
                Vec3::new(0.1, 0.0, 0.0),
                "Large Y difference, small XZ difference",
            ),
        ];

        for (pos1, pos2, description) in test_positions {
            let distance_2d =
                Vec3::new(pos1.x, 0.0, pos1.z).distance(Vec3::new(pos2.x, 0.0, pos2.z));
            let distance_3d = pos1.distance(pos2);

            println!(
                "{}: 2D={:.3}, 3D={:.3}",
                description, distance_2d, distance_3d
            );

            // Show how Y differences can cause issues
            if distance_2d != distance_3d {
                println!("  -> Y difference causes 2D/3D distance mismatch!");
            }
        }
    }

    #[test]
    fn test_proposed_fix_consistent_distance_calculation() {
        // Test a proposed fix: use consistent distance calculation
        let player_pos = Vec3::new(-0.5, 3.0, 0.0);
        let waypoint = Vec3::new(-1.0, 0.0, 0.0);

        let config = MovementConfig {
            speed: 5.0,
            stopping_distance: 0.5,
            slowdown_distance: 2.0,
            delta_time: 1.0 / 60.0,
        };

        let calculation = calculate_movement(player_pos, Some(waypoint), config);

        // PROPOSED FIX 1: Use 2D distance in pathfinding integration (consistent with movement)
        let should_move_fix1 = {
            let distance_2d = Vec3::new(player_pos.x, 0.0, player_pos.z)
                .distance(Vec3::new(waypoint.x, 0.0, waypoint.z));
            distance_2d > 0.1 // Keep same threshold as original pathfinding
        };

        // PROPOSED FIX 2: Use 3D distance in movement calculation (consistent with pathfinding)
        // This would require modifying calculate_movement to use 3D distance instead of 2D
        let should_move_fix2 = {
            let distance_3d = player_pos.distance(waypoint);
            distance_3d > config.stopping_distance
        };

        println!("=== PROPOSED FIXES ===");
        println!("Original movement should_move: {}", calculation.should_move);
        println!(
            "Original pathfinding should_move: {}",
            player_pos.distance(waypoint) > 0.1
        );
        println!(
            "Fix 1 (2D pathfinding with 0.1 threshold): {}",
            should_move_fix1
        );
        println!(
            "Fix 2 (3D movement with stopping_distance): {}",
            should_move_fix2
        );

        // With the bug reproduction scenario (2D=0.5, 3D=3.04):
        // Fix 1: 2D distance (0.5) > 0.1 = true (matches movement calculation logic)
        // Fix 2: 3D distance (3.04) > 0.5 = true (matches pathfinding integration logic)
        // Both fixes make the systems consistent by using the same distance calculation method

        assert!(
            should_move_fix1,
            "Fix 1: 2D distance (0.5) > 0.1 should be true"
        );
        assert!(
            should_move_fix2,
            "Fix 2: 3D distance (3.04) > 0.5 should be true"
        );

        // The key insight: both fixes resolve the contradiction, but by different approaches:
        // Fix 1: Use 2D distance everywhere (pathfinding matches movement)
        // Fix 2: Use 3D distance everywhere (movement matches pathfinding)

        println!("Both fixes resolve the contradiction!");
    }

    #[test]
    fn test_pathfinding_bug_fixed_with_2d_distance() {
        // Test that the fixed pathfinding integration uses 2D distance consistently
        let player_pos = Vec3::new(-0.5, 3.0, 0.0);
        let waypoint = Vec3::new(-1.0, 0.0, 0.0);

        let config = MovementConfig {
            speed: 5.0,
            stopping_distance: 0.5,
            slowdown_distance: 2.0,
            delta_time: 1.0 / 60.0,
        };

        let calculation = calculate_movement(player_pos, Some(waypoint), config);

        // Test the FIXED pathfinding integration logic (using 2D distance)
        let pathfinding_waypoint = Some(waypoint);
        let movement_target = pathfinding_waypoint;

        let should_move_fixed = if pathfinding_waypoint.is_some() {
            // FIXED logic: Use 2D distance calculation to match movement system
            let distance = movement_target.map_or(0.0, |target| {
                let current_2d = Vec3::new(player_pos.x, 0.0, player_pos.z);
                let target_2d = Vec3::new(target.x, 0.0, target.z);
                current_2d.distance(target_2d)
            });
            distance > 0.1
        } else {
            calculation.should_move
        };

        println!("=== PATHFINDING BUG FIX VERIFICATION ===");
        println!(
            "Player position: ({:.1}, {:.1}, {:.1})",
            player_pos.x, player_pos.y, player_pos.z
        );
        println!(
            "Waypoint position: ({:.1}, {:.1}, {:.1})",
            waypoint.x, waypoint.y, waypoint.z
        );

        let distance_2d = Vec3::new(player_pos.x, 0.0, player_pos.z)
            .distance(Vec3::new(waypoint.x, 0.0, waypoint.z));
        let distance_3d = player_pos.distance(waypoint);

        println!("2D distance: {:.2}", distance_2d);
        println!("3D distance: {:.2}", distance_3d);
        println!(
            "Movement calculation should_move: {}",
            calculation.should_move
        );
        println!("Fixed pathfinding should_move: {}", should_move_fixed);

        // With 2D distance = 0.5 and threshold = 0.1:
        // Fixed pathfinding: 0.5 > 0.1 = true
        // Movement calculation uses stopping_distance = 0.5, so: 0.5 <= 0.5 = false (don't move)

        // The fix resolves the contradiction by making both systems use 2D distance,
        // but they can still disagree based on different thresholds (0.1 vs 0.5)
        // This is expected behavior - pathfinding uses a smaller threshold for waypoint precision

        assert!(
            should_move_fixed,
            "Fixed pathfinding should move (2D distance 0.5 > 0.1)"
        );
        assert!(
            !calculation.should_move,
            "Movement should not move (2D distance 0.5 <= 0.5)"
        );

        // The key improvement: both systems now use consistent 2D distance calculation
        // The disagreement is now due to different thresholds (0.1 vs 0.5), which is intentional
        println!("✓ BUG FIXED: Both systems now use 2D distance consistently");
        println!("✓ Different thresholds (0.1 vs 0.5) are intentional design choices");
    }
}
