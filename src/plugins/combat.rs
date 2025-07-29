use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::{
    components::*,
    resources::*,
    game_logic::*,
};

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<CombatConfig>()
            .init_resource::<ObjectPool<Bullet>>()
            .add_systems(Update, (
                handle_combat_input,
                update_bullets,
                update_area_effects,
                bullet_enemy_collision,
                area_effect_damage,
            ));
    }
}

fn handle_combat_input(
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    combat_config: Res<CombatConfig>,
) {
    let window = windows.single();
    if let Some(cursor_pos) = window.cursor_position() {
        let (camera, camera_transform) = camera_query.single();
        
        if let Some(ray) = camera.viewport_to_world(camera_transform, cursor_pos) {
            let ground_y = 0.0;
            if ray.direction.y < 0.0 {
                let t = (ground_y - ray.origin.y) / ray.direction.y;
                let world_pos = ray.origin + ray.direction * t;
                
                // Right click: Fire bullet
                if mouse_button.just_pressed(MouseButton::Right) {
                    if let Ok(player_transform) = player_query.get_single() {
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
                                speed: combat_config.bullet_speed,
                                lifetime: combat_config.bullet_lifetime,
                                damage: combat_config.bullet_damage,
                            },
                        ));
                    }
                }
            }
        }
    }
    
    // Spacebar: Area effect
    if keyboard.just_pressed(KeyCode::Space) {
        if let Ok(player_transform) = player_query.get_single() {
            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(Cylinder::new(combat_config.area_effect_radius, 0.1)),
                    material: materials.add(StandardMaterial {
                        base_color: Color::srgba(0.5, 0.0, 1.0, 0.3),
                        alpha_mode: AlphaMode::Blend,
                        ..default()
                    }),
                    transform: Transform::from_translation(player_transform.translation),
                    ..default()
                },
                AreaEffect {
                    radius: combat_config.area_effect_radius,
                    damage_per_second: combat_config.area_effect_dps,
                    duration: combat_config.area_effect_duration,
                    elapsed: 0.0,
                },
            ));
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
    mut respawn_counter: ResMut<RespawnCounter>,
    combat_config: Res<CombatConfig>,
    enemy_config: Res<EnemyConfig>,
) {
    for (bullet_entity, bullet_transform, bullet) in bullet_query.iter() {
        for (enemy_entity, enemy_transform, mut enemy) in enemy_query.iter_mut() {
            if check_collision(
                bullet_transform.translation,
                enemy_transform.translation,
                combat_config.collision_distance,
            ) {
                // Damage enemy
                enemy.health -= bullet.damage;
                
                // Remove bullet
                commands.entity(bullet_entity).despawn();
                
                // Kill enemy if health depleted
                if enemy.health <= 0 && !enemy.is_dying {
                    enemy.is_dying = true;
                    commands.entity(enemy_entity).despawn();
                    
                    // Respawn enemy at random position
                    let respawn_pos = generate_respawn_position(
                        respawn_counter.count,
                        enemy_config.spawn_distance_min,
                        enemy_config.spawn_distance_max,
                    );
                    respawn_counter.count += 1;
                    
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
                            speed: enemy_config.speed,
                            health: enemy_config.health,
                            chase_distance: enemy_config.chase_distance,
                            is_dying: false,
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
    mut respawn_counter: ResMut<RespawnCounter>,
    _combat_config: Res<CombatConfig>,
    enemy_config: Res<EnemyConfig>,
    time: Res<Time>,
) {
    for (effect_transform, effect) in effect_query.iter() {
        for (enemy_entity, enemy_transform, mut enemy) in enemy_query.iter_mut() {
            if let Some(damage) = calculate_area_damage(
                effect.damage_per_second,
                time.delta_seconds(),
                enemy_transform.translation,
                effect_transform.translation,
                effect.radius,
            ) {
                if !enemy.is_dying {
                    enemy.health -= damage;
                    
                    // Kill enemy if health depleted
                    if enemy.health <= 0 && !enemy.is_dying {
                        enemy.is_dying = true;
                        commands.entity(enemy_entity).despawn();
                        
                        // Respawn enemy at random position
                        let respawn_pos = generate_respawn_position(
                            respawn_counter.count,
                            enemy_config.spawn_distance_min,
                            enemy_config.spawn_distance_max,
                        );
                        respawn_counter.count += 1;
                        
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
                                speed: enemy_config.speed,
                                health: enemy_config.health,
                                chase_distance: enemy_config.chase_distance,
                                is_dying: false,
                            },
                        ));
                    }
                }
            }
        }
    }
}