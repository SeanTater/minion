use crate::components::*;
use crate::game_logic::{calculate_movement, ray_to_ground_target, validate_target, MovementConfig};
use crate::map::MapDefinition;
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
                move_player.after(handle_player_input), 
                update_player_from_controller_output.after(move_player),
                debug_player_state.after(update_player_from_controller_output)
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
        // Load all LOD levels for player
        let high_scene = asset_server.load("players/hooded-high.glb#Scene0");
        let med_scene = asset_server.load("players/hooded-med.glb#Scene0");
        let low_scene = asset_server.load("players/hooded-low.glb#Scene0");

        // Determine starting LOD level based on global max setting
        let (starting_scene, starting_level) = match game_config.settings.max_lod_level.as_str() {
            "medium" => (med_scene.clone(), LodLevel::Medium),
            "low" => (low_scene.clone(), LodLevel::Low),
            _ => (high_scene.clone(), LodLevel::High),
        };

        // Get spawn position from map
        info!(
            "Player spawning at map position: ({}, {}, {})",
            map.player_spawn.x, map.player_spawn.y, map.player_spawn.z
        );

        // Use spawn position from map - spawn high above terrain to avoid intersection
        let spawn_position = Vec3::new(map.player_spawn.x, map.player_spawn.y + 5.0, map.player_spawn.z); // +5.0 to spawn well above terrain
        
        info!(
            "Player spawning at position: ({:.2}, {:.2}, {:.2}) - character controller will snap to terrain",
            spawn_position.x, spawn_position.y, spawn_position.z
        );

        // Spawn player with 3D model (scaled to 2m tall, rotated to face forward)
        info!("Spawning player with KinematicCharacterController at: ({:.2}, {:.2}, {:.2})", 
              spawn_position.x, spawn_position.y, spawn_position.z);
        let player_entity = commands.spawn((
            SceneRoot(starting_scene),
            Transform::from_translation(spawn_position).with_scale(Vec3::splat(2.0)),
            RigidBody::KinematicPositionBased,
            Collider::capsule_y(1.0, 0.5), // 2m tall capsule (1m radius + 2*0.5m hemispheres), 0.5m radius
            KinematicCharacterController {
                snap_to_ground: Some(CharacterLength::Absolute(2.0)), // Match working example
                offset: CharacterLength::Absolute(0.01), // Match working example - smaller gap
                max_slope_climb_angle: 45.0_f32.to_radians(),
                min_slope_slide_angle: 30.0_f32.to_radians(),
                slide: true, // Enable sliding on slopes
                apply_impulse_to_dynamic_bodies: true, // Better physics interaction
                ..default()
            },
            Player {
                move_target: None,
                speed: Speed::new(game_config.settings.player_movement_speed),
                health: HealthPool::new_full(game_config.settings.player_max_health),
                mana: ManaPool::new_full(game_config.settings.player_max_mana),
                energy: EnergyPool::new_full(game_config.settings.player_max_energy),
            },
            LodEntity {
                current_level: starting_level,
                high_handle: high_scene.clone(),
                med_handle: med_scene.clone(),
                low_handle: low_scene.clone(),
                entity_type: LodEntityType::Player,
            },
        )).id();
        
        info!("Player entity spawned with ID: {:?}", player_entity);
    }
}

fn handle_player_input(
    mut player_query: Query<(&Transform, &mut Player)>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
) {
    if mouse_button.just_pressed(MouseButton::Left) {
        let window = windows.single()
            .expect("Primary window should always exist when game is running");
        if let Some(cursor_pos) = window.cursor_position() {
            let (camera, camera_transform) = camera_query.single()
                .expect("Camera3d should always exist when game is running");

            if let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) {
                for (player_transform, mut player) in player_query.iter_mut() {
                    let player_y = player_transform.translation.y;
                    if let Some(target) = ray_to_ground_target(ray.origin, *ray.direction, player_y) {
                        if validate_target(player_transform.translation, target) {
                            player.move_target = Some(target);
                            info!("Target set: ({:.2}, {:.2}, {:.2})", target.x, target.y, target.z);
                        } else {
                            warn!("Invalid target: ({:.2}, {:.2}, {:.2})", target.x, target.y, target.z);
                        }
                    } else {
                        warn!("Could not calculate ground target");
                    }
                }
            } else {
                warn!("Could not get ray from camera");
            }
        }
    }
}

fn move_player(
    mut player_query: Query<(&mut Transform, &mut Player, &mut KinematicCharacterController)>,
    game_config: Res<GameConfig>,
    time: Res<Time>,
) {
    
    for (mut transform, mut player, mut controller) in player_query.iter_mut() {
        let config = MovementConfig {
            speed: player.speed.0,
            stopping_distance: game_config.settings.player_stopping_distance,
            slowdown_distance: game_config.settings.player_slowdown_distance,
            delta_time: time.delta_secs(),
        };

        let calculation = calculate_movement(transform.translation, player.move_target, config);

        if calculation.should_move {
            // Add gravity component to movement
            let movement_with_gravity = Vec3::new(
                calculation.movement_vector.x, 
                -3.0 * time.delta_secs(), 
                calculation.movement_vector.z
            );
            controller.translation = Some(movement_with_gravity);

            debug!("Moving: distance={:.2} speed_factor={:.2}", 
                   calculation.distance_to_target, calculation.slowdown_factor);

            // Handle rotation using calculation result
            if let Some(rotation_target) = calculation.rotation_target {
                transform.look_at(rotation_target, Vec3::Y);
            }
        } else {
            // Clear target if we reached it
            if player.move_target.is_some() {
                player.move_target = None;
                info!("Target reached, stopping movement (distance {:.2} <= stopping distance {:.2})", 
                      calculation.distance_to_target, game_config.settings.player_stopping_distance);
            }
            // Apply gravity when stationary
            controller.translation = Some(Vec3::new(0.0, -3.0 * time.delta_secs(), 0.0));
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

fn debug_player_state(
    player_query: Query<(&Transform, &Player), With<Player>>,
    time: Res<Time>,
) {
    // Only log every 3 seconds to avoid spam
    if (time.elapsed_secs() as u32) % 3 == 0 && time.delta_secs() < 0.02 {
        for (transform, player) in player_query.iter() {
            if let Some(target) = player.move_target {
                debug!("Player: pos=({:.1}, {:.1}, {:.1}) -> target=({:.1}, {:.1}, {:.1})",
                       transform.translation.x, transform.translation.y, transform.translation.z,
                       target.x, target.y, target.z);
            }
        }
    }
}
