use crate::components::*;
use crate::resources::{GameConfig, GameState};
use bevy::prelude::Camera3d;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_rapier3d::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_systems(
                Update,
                (handle_player_input, move_player).run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnExit(GameState::Playing), cleanup_player);
    }
}

fn spawn_player(
    mut commands: Commands,
    game_config: Res<GameConfig>,
    player_query: Query<&Player>,
    asset_server: Res<AssetServer>,
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
        
        // Spawn player with 3D model (scaled to 2m tall, rotated to face forward)
        commands.spawn((
            SceneRoot(starting_scene),
            Transform::from_xyz(0.0, 1.0, 0.0)
                .with_scale(Vec3::splat(2.0))
                .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
            RigidBody::Dynamic,
            Collider::capsule_y(1.0, 0.5), // 2m tall capsule (1m radius + 2*0.5m hemispheres), 0.5m radius
            LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z, // Prevent tipping over
            Friction::coefficient(0.7),
            Restitution::coefficient(0.0), // No bouncing
            ColliderMassProperties::Density(1.0),
            ExternalForce::default(),
            Velocity::default(),
            Damping { linear_damping: 3.0, angular_damping: 8.0 }, // Increased damping for stability
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
        ));
    }
}

fn handle_player_input(
    mut player_query: Query<&mut Player>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
) {
    if mouse_button.just_pressed(MouseButton::Left) {
        let Ok(window) = windows.single() else {
            return;
        };
        if let Some(cursor_pos) = window.cursor_position() {
            let Ok((camera, camera_transform)) = camera_query.single() else {
                return;
            };

            if let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) {
                let ground_y = 0.0;
                if ray.direction.y < 0.0 {
                    let t = (ground_y - ray.origin.y) / ray.direction.y;
                    let hit_point = ray.origin + ray.direction * t;

                    for mut player in player_query.iter_mut() {
                        let target = Vec3::new(hit_point.x, 1.0, hit_point.z);
                        player.move_target = Some(target);
                    }
                }
            }
        }
    }
}

fn move_player(
    mut player_query: Query<(&Transform, &mut Player, &mut ExternalForce, &mut Velocity)>,
    game_config: Res<GameConfig>,
) {
    for (transform, mut player, mut ext_force, velocity) in player_query.iter_mut() {
        if let Some(target) = player.move_target {
            // Only use X and Z for movement calculation (ignore Y differences)
            let player_pos_2d = Vec3::new(transform.translation.x, 0.0, transform.translation.z);
            let target_2d = Vec3::new(target.x, 0.0, target.z);
            let direction = (target_2d - player_pos_2d).normalize_or_zero();
            let distance = player_pos_2d.distance(target_2d);

            if distance > game_config.settings.player_stopping_distance {
                // Calculate desired velocity toward target
                let max_speed = player.speed.0;
                let desired_velocity = direction * max_speed;
                
                // Smoothly blend current velocity toward desired velocity
                let current_velocity = Vec3::new(velocity.linvel.x, 0.0, velocity.linvel.z);
                let velocity_diff = desired_velocity - current_velocity;
                
                // Apply force to achieve desired velocity, but damped
                let acceleration_force = velocity_diff * game_config.settings.player_acceleration_force;
                ext_force.force = Vec3::new(acceleration_force.x, 0.0, acceleration_force.z);
                
                // Slow down as we approach the target
                let slowdown_factor = (distance / game_config.settings.player_slowdown_distance).min(1.0);
                ext_force.force *= slowdown_factor;
                
                // Apply torque for rotation toward movement direction
                if direction.length() > 0.1 {
                    // Calculate target yaw angle - FIXED: add PI/2 to face forward correctly
                    let target_yaw = direction.z.atan2(direction.x) + std::f32::consts::FRAC_PI_2;
                    let current_yaw = transform.rotation.to_euler(EulerRot::YXZ).0;
                    
                    // Calculate shortest rotation difference
                    let mut yaw_diff = target_yaw - current_yaw;
                    
                    // Normalize to [-π, π] range for shortest rotation
                    while yaw_diff > std::f32::consts::PI {
                        yaw_diff -= 2.0 * std::f32::consts::PI;
                    }
                    while yaw_diff < -std::f32::consts::PI {
                        yaw_diff += 2.0 * std::f32::consts::PI;
                    }
                    
                    // Apply torque proportional to the angle difference
                    ext_force.torque = Vec3::new(0.0, yaw_diff * game_config.settings.player_rotation_torque, 0.0);
                }
            } else {
                player.move_target = None;
                
                // Actively brake to a stop
                let current_velocity = Vec3::new(velocity.linvel.x, 0.0, velocity.linvel.z);
                ext_force.force = -current_velocity * game_config.settings.player_braking_force;
                ext_force.torque = Vec3::ZERO;
            }
        } else {
            // Stop movement when no target - apply braking
            let current_velocity = Vec3::new(velocity.linvel.x, 0.0, velocity.linvel.z);
            ext_force.force = -current_velocity * (game_config.settings.player_braking_force * 0.75);
            ext_force.torque = Vec3::ZERO;
        }
    }
}

fn cleanup_player(mut commands: Commands, player_query: Query<Entity, With<Player>>) {
    for entity in player_query.iter() {
        commands.entity(entity).despawn();
    }
}
