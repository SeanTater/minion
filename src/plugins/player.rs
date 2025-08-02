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
            Transform::from_xyz(0.0, 2.0, 0.0)
                .with_scale(Vec3::splat(2.0)),
            RigidBody::KinematicPositionBased,
            Collider::capsule_y(1.0, 0.5), // 2m tall capsule (1m radius + 2*0.5m hemispheres), 0.5m radius
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
    mut player_query: Query<(&mut Transform, &mut Player)>,
    game_config: Res<GameConfig>,
    time: Res<Time>,
) {
    for (mut transform, mut player) in player_query.iter_mut() {
        if let Some(target) = player.move_target {
            // Only use X and Z for movement calculation (ignore Y differences)
            let player_pos_2d = Vec3::new(transform.translation.x, 0.0, transform.translation.z);
            let target_2d = Vec3::new(target.x, 0.0, target.z);
            let direction = (target_2d - player_pos_2d).normalize_or_zero();
            let distance = player_pos_2d.distance(target_2d);

            if distance > game_config.settings.player_stopping_distance {
                // Direct kinematic movement
                let max_speed = player.speed.0;
                let move_distance = max_speed * time.delta_secs();
                
                // Slow down as we approach the target
                let slowdown_factor = (distance / game_config.settings.player_slowdown_distance).min(1.0);
                let actual_move_distance = move_distance * slowdown_factor;
                
                // Move toward target, clamped to not overshoot
                let movement = direction * actual_move_distance.min(distance);
                transform.translation += movement;
                
                // Rotate toward movement direction
                // NOTE: GLB models are facing backwards, so we flip the direction
                if direction.length() > 0.1 {
                    let character_pos = transform.translation;
                    let flat_target = Vec3::new(
                        character_pos.x - direction.x, // Flip for GLB orientation
                        character_pos.y, // Keep same Y level
                        character_pos.z - direction.z  // Flip for GLB orientation
                    );
                    transform.look_at(flat_target, Vec3::Y);
                }
            } else {
                player.move_target = None;
            }
        }
    }
}

fn cleanup_player(mut commands: Commands, player_query: Query<Entity, With<Player>>) {
    for entity in player_query.iter() {
        commands.entity(entity).despawn();
    }
}
