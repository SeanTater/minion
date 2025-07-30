use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::prelude::Camera3d;
use crate::components::*;
use crate::resources::{GameState, GameConfig};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_systems(Update, (handle_player_input, move_player).run_if(in_state(GameState::Playing)));
    }
}

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    game_config: Res<GameConfig>,
) {
    // Player character (simple capsule)
    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.5, 2.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.2, 0.2),
            ..default()
        })),
        Transform::from_xyz(0.0, 1.0, 0.0),
        Player {
            move_target: None,
            speed: Speed::new(game_config.settings.player_movement_speed),
            health: HealthPool::new_full(game_config.settings.player_max_health),
            mana: ManaPool::new_full(game_config.settings.player_max_mana),
            energy: EnergyPool::new_full(game_config.settings.player_max_energy),
        },
    ));
}

fn handle_player_input(
    mut player_query: Query<&mut Player>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
) {
    if mouse_button.just_pressed(MouseButton::Left) {
        let Ok(window) = windows.single() else { return; };
        if let Some(cursor_pos) = window.cursor_position() {
            let Ok((camera, camera_transform)) = camera_query.single() else { return; };
            
            if let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) {
                let ground_y = 0.0;
                if ray.direction.y < 0.0 {
                    let t = (ground_y - ray.origin.y) / ray.direction.y;
                    let hit_point = ray.origin + ray.direction * t;
                    
                    for mut player in player_query.iter_mut() {
                        player.move_target = Some(Vec3::new(hit_point.x, 1.0, hit_point.z));
                    }
                }
            }
        }
    }
}

fn move_player(
    mut player_query: Query<(&mut Transform, &mut Player)>,
    time: Res<Time>,
) {
    for (mut transform, mut player) in player_query.iter_mut() {
        if let Some(target) = player.move_target {
            let direction = (target - transform.translation).normalize_or_zero();
            let distance = transform.translation.distance(target);
            
            if distance > 0.1 {
                transform.translation += direction * player.speed * time.delta_secs();
                
                // Face movement direction
                if direction.length() > 0.1 {
                    transform.look_to(direction, Vec3::Y);
                }
            } else {
                player.move_target = None;
            }
        }
    }
}