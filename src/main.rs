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

#[derive(Component)]
struct Bullet {
    direction: Vec3,
    speed: f32,
    lifetime: f32,
    damage: i32,
}

#[derive(Component)]
struct AreaEffect {
    radius: f32,
    damage_per_second: i32,
    duration: f32,
    elapsed: f32,
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
        .init_resource::<ObjectPool<Bullet>>()
        .add_systems(Startup, (setup_scene, spawn_player, spawn_enemies))
        .add_systems(Update, (
            handle_input,
            move_player,
            follow_camera,
            enemy_ai,
            update_bullets,
            update_area_effects,
            bullet_enemy_collision,
            area_effect_damage,
        ))
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

fn handle_input(
    mut commands: Commands,
    mut player_query: Query<(&Transform, &mut Player)>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let window = windows.single();
    if let Some(cursor_pos) = window.cursor_position() {
        let (camera, camera_transform) = camera_query.single();
        
        if let Some(ray) = camera.viewport_to_world(camera_transform, cursor_pos) {
            let ground_y = 0.0;
            if ray.direction.y < 0.0 {
                let t = (ground_y - ray.origin.y) / ray.direction.y;
                let world_pos = ray.origin + ray.direction * t;
                
                // Left click: Move
                if mouse_button.just_pressed(MouseButton::Left) {
                    for (_, mut player) in player_query.iter_mut() {
                        player.move_target = Some(Vec3::new(world_pos.x, 1.0, world_pos.z));
                    }
                }
                
                // Right click: Fire bullet
                if mouse_button.just_pressed(MouseButton::Right) {
                    if let Ok((player_transform, _)) = player_query.get_single() {
                        let direction = (world_pos - player_transform.translation).normalize();
                        
                        commands.spawn((
                            PbrBundle {
                                mesh: meshes.add(Sphere::new(0.1)),
                                material: materials.add(StandardMaterial {
                                    base_color: Color::srgb(1.0, 1.0, 0.0),
                                    emissive: Color::srgb(0.5, 0.5, 0.0).into(),
                                    ..default()
                                }),
                                transform: Transform::from_translation(player_transform.translation + Vec3::Y * 0.5),
                                ..default()
                            },
                            Bullet {
                                direction: Vec3::new(direction.x, 0.0, direction.z).normalize(),
                                speed: 15.0,
                                lifetime: 3.0,
                                damage: 2,
                            },
                        ));
                    }
                }
            }
        }
    }
    
    // Spacebar: Area effect
    if keyboard.just_pressed(KeyCode::Space) {
        if let Ok((player_transform, _)) = player_query.get_single() {
            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(Cylinder::new(3.0, 0.1)),
                    material: materials.add(StandardMaterial {
                        base_color: Color::srgba(0.5, 0.0, 1.0, 0.3),
                        alpha_mode: AlphaMode::Blend,
                        ..default()
                    }),
                    transform: Transform::from_translation(player_transform.translation),
                    ..default()
                },
                AreaEffect {
                    radius: 3.0,
                    damage_per_second: 5,
                    duration: 2.0,
                    elapsed: 0.0,
                },
            ));
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


fn update_bullets(
    mut commands: Commands,
    mut bullet_query: Query<(Entity, &mut Transform, &mut Bullet)>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut bullet) in bullet_query.iter_mut() {
        // Move bullet
        transform.translation += bullet.direction * bullet.speed * time.delta_seconds();
        
        // Update lifetime
        bullet.lifetime -= time.delta_seconds();
        
        // Despawn when lifetime expires
        if bullet.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn update_area_effects(
    mut commands: Commands,
    mut effect_query: Query<(Entity, &mut AreaEffect, &mut Transform)>,
    time: Res<Time>,
) {
    for (entity, mut effect, mut transform) in effect_query.iter_mut() {
        effect.elapsed += time.delta_seconds();
        
        // Fade effect over time
        let alpha = 1.0 - (effect.elapsed / effect.duration);
        transform.scale = Vec3::splat(alpha.max(0.1));
        
        // Despawn when duration expires
        if effect.elapsed >= effect.duration {
            commands.entity(entity).despawn();
        }
    }
}

fn bullet_enemy_collision(
    mut commands: Commands,
    bullet_query: Query<(Entity, &Transform, &Bullet), With<Bullet>>,
    mut enemy_query: Query<(Entity, &Transform, &mut Enemy), (With<Enemy>, Without<Bullet>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    for (bullet_entity, bullet_transform, bullet) in bullet_query.iter() {
        for (enemy_entity, enemy_transform, mut enemy) in enemy_query.iter_mut() {
            let distance = bullet_transform.translation.distance(enemy_transform.translation);
            
            if distance <= 0.6 { // Collision threshold
                // Damage enemy
                enemy.health -= bullet.damage;
                
                // Remove bullet
                commands.entity(bullet_entity).despawn();
                
                // Kill enemy if health depleted
                if enemy.health <= 0 {
                    commands.entity(enemy_entity).despawn();
                    
                    // Respawn enemy
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
                break;
            }
        }
    }
}

fn area_effect_damage(
    mut commands: Commands,
    effect_query: Query<(&Transform, &AreaEffect)>,
    mut enemy_query: Query<(Entity, &Transform, &mut Enemy), (With<Enemy>, Without<AreaEffect>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    for (effect_transform, effect) in effect_query.iter() {
        for (enemy_entity, enemy_transform, mut enemy) in enemy_query.iter_mut() {
            let distance = effect_transform.translation.distance(enemy_transform.translation);
            
            if distance <= effect.radius {
                // Distance-based damage falloff
                let damage_multiplier = (1.0 - (distance / effect.radius)).max(0.0);
                let damage = (effect.damage_per_second as f32 * damage_multiplier * time.delta_seconds()) as i32;
                
                if damage > 0 {
                    enemy.health -= damage.max(1);
                    
                    // Kill enemy if health depleted
                    if enemy.health <= 0 {
                        commands.entity(enemy_entity).despawn();
                        
                        // Respawn enemy
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
}
