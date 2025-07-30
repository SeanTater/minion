use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::components::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, spawn_player)
            .add_systems(Update, (handle_player_input, move_player));
    }
}

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Player character (simple capsule)
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Capsule3d::new(0.5, 2.0)),
            material: materials.add(StandardMaterial {
                base_color: Color::srgb(0.8, 0.2, 0.2),
                ..default()
            }),
            transform: Transform::from_xyz(0.0, 1.0, 0.0),
            ..default()
        },
        Player {
            move_target: None,
            speed: 5.0,
        },
    ));
}

fn handle_player_input(
    mut player_query: Query<&mut Player>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
) {
    if mouse_button.just_pressed(MouseButton::Left) {
        let window = windows.single();
        if let Some(cursor_pos) = window.cursor_position() {
            let (camera, camera_transform) = camera_query.single();
            
            if let Some(ray) = camera.viewport_to_world(camera_transform, cursor_pos) {
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
                transform.translation += direction * player.speed * time.delta_seconds();
                
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