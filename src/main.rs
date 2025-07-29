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

#[derive(Component)]
struct Enemy {
    speed: f32,
    health: i32,
    chase_distance: f32,
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
        .init_resource::<ObjectPool<Enemy>>()
        .add_systems(Startup, (setup_scene, spawn_player, spawn_enemies))
        .add_systems(Update, (handle_mouse_clicks, move_player, follow_camera, enemy_ai, combat_system))
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

fn spawn_enemies(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let spawn_positions = [
        Vec3::new(5.0, 1.0, 5.0),
        Vec3::new(-5.0, 1.0, 5.0),  
        Vec3::new(5.0, 1.0, -5.0),
        Vec3::new(-5.0, 1.0, -5.0),
        Vec3::new(0.0, 1.0, 8.0),
    ];

    for pos in spawn_positions {
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Sphere::new(0.5)),
                material: materials.add(StandardMaterial {
                    base_color: Color::srgb(0.8, 0.1, 0.1),
                    ..default()
                }),
                transform: Transform::from_translation(pos),
                ..default()
            },
            Enemy {
                speed: 3.0,
                health: 3,
                chase_distance: 8.0,
            },
        ));
    }
}

fn enemy_ai(
    mut enemy_query: Query<(&mut Transform, &Enemy), (With<Enemy>, Without<Player>)>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    time: Res<Time>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        for (mut enemy_transform, enemy) in enemy_query.iter_mut() {
            let distance = enemy_transform.translation.distance(player_transform.translation);
            
            if distance <= enemy.chase_distance && distance > 1.0 {
                let direction = (player_transform.translation - enemy_transform.translation).normalize();
                enemy_transform.translation += direction * enemy.speed * time.delta_seconds();
                enemy_transform.look_to(direction, Vec3::Y);
            }
        }
    }
}

fn combat_system(
    mut commands: Commands,
    mut enemy_query: Query<(Entity, &mut Transform, &mut Enemy), With<Enemy>>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        for (entity, enemy_transform, mut enemy) in enemy_query.iter_mut() {
            let distance = enemy_transform.translation.distance(player_transform.translation);
            
            if distance <= 1.2 {
                enemy.health -= 1;
                
                if enemy.health <= 0 {
                    commands.entity(entity).despawn();
                    
                    // Respawn enemy at random position
                    let respawn_pos = Vec3::new(
                        (time.elapsed_seconds().sin() * 8.0) as f32,
                        1.0,
                        (time.elapsed_seconds().cos() * 8.0) as f32,
                    );
                    
                    commands.spawn((
                        PbrBundle {
                            mesh: meshes.add(Sphere::new(0.5)),
                            material: materials.add(StandardMaterial {
                                base_color: Color::srgb(0.8, 0.1, 0.1),
                                ..default()
                            }),
                            transform: Transform::from_translation(respawn_pos),
                            ..default()
                        },
                        Enemy {
                            speed: 3.0,
                            health: 3,
                            chase_distance: 8.0,
                        },
                    ));
                }
            }
        }
    }
}
