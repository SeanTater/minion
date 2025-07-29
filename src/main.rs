use bevy::prelude::*;
use bevy::window::PrimaryWindow;

#[derive(Component)]
struct Player {
    move_target: Option<Vec3>,
    speed: f32,
}

#[derive(Component)]
struct Ground;

#[derive(Component)]
struct CameraFollow {
    offset: Vec3,
}

#[derive(Resource)]
struct ObjectPool<T: Component> {
    available: Vec<Entity>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Component> Default for ObjectPool<T> {
    fn default() -> Self {
        Self {
            available: Vec::new(),
            _phantom: std::marker::PhantomData,
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Minion - Diablo-like Game".into(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, (setup_scene, spawn_player))
        .add_systems(Update, (handle_mouse_clicks, move_player, follow_camera))
        .run();
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Ground plane
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Plane3d::default().mesh().size(20.0, 20.0)),
            material: materials.add(StandardMaterial {
                base_color: Color::srgb(0.3, 0.5, 0.3),
                ..default()
            }),
            ..default()
        },
        Ground,
    ));

    // Light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
            ..default()
        },
        ..default()
    });

    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 300.0,
    });

    // Camera with isometric view that follows player
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(10.0, 15.0, 10.0)
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        CameraFollow {
            offset: Vec3::new(10.0, 15.0, 10.0),
        },
    ));
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

fn handle_mouse_clicks(
    mut player_query: Query<&mut Player>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    _ground_query: Query<&GlobalTransform, (With<Ground>, Without<Camera>)>,
) {
    if mouse_button.just_pressed(MouseButton::Left) {
        let window = windows.single();
        if let Some(cursor_pos) = window.cursor_position() {
            let (camera, camera_transform) = camera_query.single();
            
            if let Some(ray) = camera.viewport_to_world(camera_transform, cursor_pos) {
                // Cast ray to ground plane (y = 0)
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

fn follow_camera(
    player_query: Query<&Transform, (With<Player>, Without<CameraFollow>)>,
    mut camera_query: Query<(&mut Transform, &CameraFollow), Without<Player>>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        for (mut camera_transform, follow) in camera_query.iter_mut() {
            let target_pos = player_transform.translation + follow.offset;
            camera_transform.translation = target_pos;
            camera_transform.look_at(player_transform.translation, Vec3::Y);
        }
    }
}
