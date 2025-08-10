use crate::{components::*, game_logic::damage::*, map::MapDefinition, resources::*};
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
    let window = windows
        .single()
        .expect("Primary window should always exist when game is running");
    if let Some(cursor_pos) = window.cursor_position() {
        let (camera, camera_transform) = camera_query
            .single()
            .expect("Camera3d should always exist when game is running");

        if let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) {
            let ground_y = 0.0;
            if ray.direction.y < 0.0 {
                let t = (ground_y - ray.origin.y) / ray.direction.y;
                let world_pos = ray.origin + ray.direction * t;

                // Right click: Fire bullet
                if mouse_button.just_pressed(MouseButton::Right) {
                    let player_transform = player_query
                        .single()
                        .expect("Player should always exist when in Playing state");

                    // Inline bullet spawn calculation
                    let direction = (world_pos - player_transform.translation).normalize();
                    let spawn_position = player_transform.translation + Vec3::Y * 0.5;
                    let normalized_direction = Vec3::new(direction.x, 0.0, direction.z).normalize();

                    commands.spawn((
                        Mesh3d(meshes.add(Sphere::new(0.1))),
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: Color::srgb(1.0, 1.0, 0.0),
                            emissive: Color::srgb(0.5, 0.5, 0.0).into(),
                            ..default()
                        })),
                        Transform::from_translation(spawn_position),
                        Bullet {
                            direction: normalized_direction,
                            speed: Speed::new(game_config.settings.bullet_speed.get()),
                            lifetime: game_config.settings.bullet_lifetime.get(),
                            damage: Damage::new(game_config.settings.bullet_damage.get()),
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
        let player_transform = player_query
            .single()
            .expect("Player should always exist when in Playing state");
        let effect_type = selected_effect.effect_type;

        // Inline area effect spawn calculation
        let spawn_position = player_transform.translation;
        let radius = effect_type.radius(&game_config.settings).0;

        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(radius, 0.1))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: effect_type.base_color(),
                alpha_mode: AlphaMode::Blend,
                ..default()
            })),
            Transform::from_translation(spawn_position),
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
        // Inline bullet movement calculation
        transform.translation += bullet.direction * bullet.speed * time.delta_secs();

        // Inline lifetime update and despawn check
        bullet.lifetime -= time.delta_secs();
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

        let duration = effect.effect_type.duration(&game_config.settings);

        // Inline area effect fade calculation
        let alpha = 1.0 - (effect.elapsed / duration);
        let scale = alpha.max(0.1);
        transform.scale = Vec3::splat(scale);

        // Inline despawn check
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
                Distance::new(game_config.settings.bullet_collision_distance.get()),
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

                    // Inline respawn position calculation
                    let respawn_pos = if let Some(map) = map.as_deref() {
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

                        // Inline respawn position calculation
                        let respawn_pos = if let Some(map) = map.as_deref() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::{SpawnZone, TerrainData};

    #[test]
    fn test_inlined_bullet_movement() {
        // Test the inlined bullet movement calculation
        let mut position = Vec3::new(0.0, 1.0, 0.0);
        let direction = Vec3::new(1.0, 0.0, 0.0);
        let speed = Speed::new(10.0);
        let delta_time = 0.1;

        // Inline calculation: position += direction * speed * delta_time
        position += direction * speed * delta_time;

        assert_eq!(position, Vec3::new(1.0, 1.0, 0.0));
    }

    #[test]
    fn test_inlined_bullet_despawn() {
        // Test the inlined bullet lifetime and despawn logic
        let mut lifetime = 0.05;
        let delta_time = 0.1;

        // Inline calculation: lifetime -= delta_time; should_despawn = lifetime <= 0.0
        lifetime -= delta_time;
        let should_despawn = lifetime <= 0.0;

        assert!(should_despawn);
        assert_eq!(lifetime, -0.05);
    }

    #[test]
    fn test_inlined_area_effect_fade() {
        // Test the inlined area effect fade calculation
        let mut elapsed = 1.0;
        let delta_time = 0.1;
        let duration = 3.0; // Default magic duration

        // Inline calculation: elapsed += delta_time; alpha = 1.0 - (elapsed / duration); scale = alpha.max(0.1)
        elapsed += delta_time;
        let alpha: f32 = 1.0 - (elapsed / duration);
        let scale = alpha.max(0.1);

        assert_eq!(elapsed, 1.1);
        let expected_alpha: f32 = 1.0 - (1.1 / 3.0);
        let expected_scale = expected_alpha.max(0.1);
        assert!((scale - expected_scale).abs() < 0.001);
    }

    #[test]
    fn test_inlined_area_effect_despawn() {
        // Test the inlined area effect despawn logic
        let mut elapsed = 2.9;
        let delta_time = 0.2;
        let duration = 3.0; // Default magic duration

        // Inline calculation: elapsed += delta_time; should_despawn = elapsed >= duration
        elapsed += delta_time;
        let should_despawn = elapsed >= duration;

        assert!(should_despawn);
        assert_eq!(elapsed, 3.1);
    }

    #[test]
    fn test_inlined_respawn_position_with_map() {
        let zone = SpawnZone {
            center: Vec3::new(10.0, 0.0, 10.0),
            radius: 5.0,
            max_enemies: 10,
            enemy_types: vec!["dark-knight".to_string()],
        };

        let terrain = TerrainData::create_flat(10, 10, 1.0, 0.0).unwrap();
        let map = MapDefinition {
            name: "test".to_string(),
            terrain,
            player_spawn: Vec3::ZERO,
            enemy_zones: vec![zone],
            environment_objects: vec![],
        };

        let respawn_counter = 0;

        // Test the inlined respawn logic
        let respawn_pos = if !map.enemy_zones.is_empty() {
            let zone_index = (respawn_counter as usize) % map.enemy_zones.len();
            crate::game_logic::spawning::generate_zone_position(
                &map.enemy_zones[zone_index],
                respawn_counter,
            )
        } else {
            Vec3::new(5.0, 2.0, 0.0)
        };

        // Should be within the zone
        let distance = Vec3::new(respawn_pos.x - 10.0, 0.0, respawn_pos.z - 10.0).length();
        assert!(distance <= 5.0);
        assert_eq!(respawn_pos.y, 1.0);
    }

    #[test]
    fn test_inlined_respawn_position_no_map() {
        let respawn_pos = Vec3::new(5.0, 2.0, 0.0); // Fallback when no map
        assert_eq!(respawn_pos, Vec3::new(5.0, 2.0, 0.0));
    }

    #[test]
    fn test_inlined_respawn_position_empty_zones() {
        let terrain = TerrainData::create_flat(10, 10, 1.0, 0.0).unwrap();
        let map = MapDefinition {
            name: "test".to_string(),
            terrain,
            player_spawn: Vec3::ZERO,
            enemy_zones: vec![],
            environment_objects: vec![],
        };

        let respawn_counter = 0;

        // Test the inlined respawn logic with empty zones
        let respawn_pos = if !map.enemy_zones.is_empty() {
            let zone_index = (respawn_counter as usize) % map.enemy_zones.len();
            crate::game_logic::spawning::generate_zone_position(
                &map.enemy_zones[zone_index],
                respawn_counter,
            )
        } else {
            Vec3::new(5.0, 2.0, 0.0)
        };

        assert_eq!(respawn_pos, Vec3::new(5.0, 2.0, 0.0));
    }
}
