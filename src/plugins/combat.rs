use crate::{components::*, game_logic::*, map::MapDefinition, resources::*};
use bevy::prelude::Camera3d;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedAreaEffect>()
            .add_systems(
                Update,
                (
                    handle_combat_input,
                    update_bullets,
                    update_area_effects,
                    bullet_enemy_collision,
                    area_effect_damage,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnExit(GameState::Playing), cleanup_combat_entities);
    }
}

fn handle_combat_input(
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    game_config: Res<GameConfig>,
    mut selected_effect: ResMut<SelectedAreaEffect>,
) {
    let window = windows.single()
        .expect("Primary window should always exist when game is running");
    if let Some(cursor_pos) = window.cursor_position() {
        let (camera, camera_transform) = camera_query.single()
            .expect("Camera3d should always exist when game is running");

        if let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) {
            let ground_y = 0.0;
            if ray.direction.y < 0.0 {
                let t = (ground_y - ray.origin.y) / ray.direction.y;
                let world_pos = ray.origin + ray.direction * t;

                // Right click: Fire bullet
                if mouse_button.just_pressed(MouseButton::Right) {
                    let player_transform = player_query.single()
                        .expect("Player should always exist when in Playing state");
                    let direction = (world_pos - player_transform.translation).normalize();

                    commands.spawn((
                        Mesh3d(meshes.add(Sphere::new(0.1))),
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: Color::srgb(1.0, 1.0, 0.0),
                            emissive: Color::srgb(0.5, 0.5, 0.0).into(),
                            ..default()
                        })),
                        Transform::from_translation(
                            player_transform.translation + Vec3::Y * 0.5,
                        ),
                        Bullet {
                            direction: Vec3::new(direction.x, 0.0, direction.z).normalize(),
                            speed: Speed::new(game_config.settings.bullet_speed),
                            lifetime: game_config.settings.bullet_lifetime,
                            damage: Damage::new(game_config.settings.bullet_damage),
                        },
                    ));
                }
            }
        }
    }

    // Tab: Cycle area effect type
    if keyboard.just_pressed(KeyCode::Tab) {
        selected_effect.effect_type = match selected_effect.effect_type {
            AreaEffectType::Magic => AreaEffectType::Poison,
            AreaEffectType::Poison => AreaEffectType::Magic,
        };
    }

    // Spacebar: Area effect
    if keyboard.just_pressed(KeyCode::Space) {
        let player_transform = player_query.single()
            .expect("Player should always exist when in Playing state");
        let effect_type = selected_effect.effect_type;
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(
                effect_type.radius(&game_config.settings).0,
                0.1,
            ))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: effect_type.base_color(),
                alpha_mode: AlphaMode::Blend,
                ..default()
            })),
            Transform::from_translation(player_transform.translation),
            AreaEffect {
                effect_type,
                elapsed: 0.0,
            },
        ));
    }
}

fn update_bullets(
    mut commands: Commands,
    mut bullet_query: Query<(Entity, &mut Transform, &mut Bullet)>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut bullet) in bullet_query.iter_mut() {
        // Move bullet
        transform.translation += bullet.direction * bullet.speed * time.delta_secs();

        // Update lifetime
        bullet.lifetime -= time.delta_secs();

        // Despawn when lifetime expires
        if bullet.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn update_area_effects(
    mut commands: Commands,
    mut effect_query: Query<(Entity, &mut AreaEffect, &mut Transform)>,
    game_config: Res<GameConfig>,
    time: Res<Time>,
) {
    for (entity, mut effect, mut transform) in effect_query.iter_mut() {
        effect.elapsed += time.delta_secs();

        // Fade effect over time
        let duration = effect.effect_type.duration(&game_config.settings);
        let alpha = 1.0 - (effect.elapsed / duration);
        transform.scale = Vec3::splat(alpha.max(0.1));

        // Despawn when duration expires
        if effect.elapsed >= duration {
            commands.entity(entity).despawn();
        }
    }
}

fn bullet_enemy_collision(
    mut commands: Commands,
    bullet_query: Query<(Entity, &Transform, &Bullet), With<Bullet>>,
    mut enemy_query: Query<(Entity, &Transform, &mut Enemy), (With<Enemy>, Without<Bullet>)>,
    asset_server: Res<AssetServer>,
    mut respawn_counter: ResMut<RespawnCounter>,
    mut game_config: ResMut<GameConfig>,
    map: Option<Res<MapDefinition>>,
) {
    for (bullet_entity, bullet_transform, bullet) in bullet_query.iter() {
        for (enemy_entity, enemy_transform, mut enemy) in enemy_query.iter_mut() {
            if check_collision(
                bullet_transform.translation,
                enemy_transform.translation,
                Distance::new(game_config.settings.bullet_collision_distance),
            ) {
                // Damage enemy
                enemy.health.take_damage(bullet.damage);

                // Remove bullet
                commands.entity(bullet_entity).despawn();

                // Kill enemy if health depleted
                if enemy.health.is_dead() && !enemy.is_dying {
                    enemy.is_dying = true;
                    game_config.score += game_config.settings.score_per_enemy;
                    commands.entity(enemy_entity).despawn();

                    // Respawn enemy in a spawn zone if map is loaded, otherwise use fallback
                    let respawn_pos = if let Some(map) = &map {
                        if !map.enemy_zones.is_empty() {
                            let zone_index =
                                (respawn_counter.count as usize) % map.enemy_zones.len();
                            crate::game_logic::spawning::generate_zone_position(
                                &map.enemy_zones[zone_index],
                                respawn_counter.count,
                            )
                        } else {
                            Vec3::new(5.0, 2.0, 0.0) // Safe fallback if no zones
                        }
                    } else {
                        Vec3::new(5.0, 2.0, 0.0) // Safe fallback if no map
                    };
                    respawn_counter.count += 1;

                    crate::game_logic::spawning::spawn_enemy_entity(
                        &mut commands,
                        &asset_server,
                        respawn_pos,
                        &game_config,
                    );
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
    asset_server: Res<AssetServer>,
    mut respawn_counter: ResMut<RespawnCounter>,
    mut game_config: ResMut<GameConfig>,
    time: Res<Time>,
    map: Option<Res<MapDefinition>>,
) {
    for (effect_transform, effect) in effect_query.iter() {
        for (enemy_entity, enemy_transform, mut enemy) in enemy_query.iter_mut() {
            if let Some(damage) = calculate_area_damage(
                effect.effect_type.damage_per_second(&game_config.settings),
                time.delta_secs(),
                enemy_transform.translation,
                effect_transform.translation,
                effect.effect_type.radius(&game_config.settings),
            ) {
                if !enemy.is_dying {
                    enemy.health.take_damage(damage);

                    // Kill enemy if health depleted
                    if enemy.health.is_dead() && !enemy.is_dying {
                        enemy.is_dying = true;
                        game_config.score += game_config.settings.score_per_enemy;
                        commands.entity(enemy_entity).despawn();

                        // Respawn enemy in a spawn zone if map is loaded, otherwise use fallback
                        let respawn_pos = if let Some(map) = &map {
                            if !map.enemy_zones.is_empty() {
                                let zone_index =
                                    (respawn_counter.count as usize) % map.enemy_zones.len();
                                crate::game_logic::spawning::generate_zone_position(
                                    &map.enemy_zones[zone_index],
                                    respawn_counter.count,
                                )
                            } else {
                                Vec3::new(5.0, 2.0, 0.0) // Safe fallback if no zones
                            }
                        } else {
                            Vec3::new(5.0, 2.0, 0.0) // Safe fallback if no map
                        };
                        respawn_counter.count += 1;

                        crate::game_logic::spawning::spawn_enemy_entity(
                            &mut commands,
                            &asset_server,
                            respawn_pos,
                            &game_config,
                        );
                    }
                }
            }
        }
    }
}

fn cleanup_combat_entities(
    mut commands: Commands,
    bullet_query: Query<Entity, With<Bullet>>,
    area_effect_query: Query<Entity, With<AreaEffect>>,
) {
    // Clean up bullets
    for entity in bullet_query.iter() {
        commands.entity(entity).despawn();
    }

    // Clean up area effects
    for entity in area_effect_query.iter() {
        commands.entity(entity).despawn();
    }
}
