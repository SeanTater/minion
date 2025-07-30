use crate::{components::*, game_logic::*, resources::*};
use bevy::prelude::*;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EnemyConfig>()
            .init_resource::<ObjectPool<Enemy>>()
            .insert_resource(RespawnCounter { count: 0 })
            .add_systems(OnEnter(GameState::Playing), spawn_enemies)
            .add_systems(Update, (enemy_ai, enemy_collision).run_if(in_state(GameState::Playing)));
    }
}

fn spawn_enemies(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    enemy_config: Res<EnemyConfig>,
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
            Mesh3d(meshes.add(Sphere::new(0.5))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.8, 0.1, 0.1),
                ..default()
            })),
            Transform::from_translation(pos),
            Enemy {
                speed: enemy_config.speed,
                health: enemy_config.health,
                chase_distance: enemy_config.chase_distance,
                is_dying: false,
            },
            Name(generate_dark_name()),
        ));
    }
}

fn enemy_ai(
    mut enemy_query: Query<(&mut Transform, &Enemy), (With<Enemy>, Without<Player>)>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    time: Res<Time>,
) {
    if let Ok(player_transform) = player_query.single() {
        for (mut enemy_transform, enemy) in enemy_query.iter_mut() {
            if enemy.is_dying {
                continue; // Skip dying enemies
            }

            let distance = enemy_transform
                .translation
                .distance(player_transform.translation);

            if distance <= enemy.chase_distance && distance > 1.0 {
                let direction =
                    (player_transform.translation - enemy_transform.translation).normalize();
                enemy_transform.translation += direction * enemy.speed * time.delta_secs();
                enemy_transform.look_to(direction, Vec3::Y);
            }
        }
    }
}

fn enemy_collision(
    mut enemy_query: Query<(Entity, &mut Transform, &Enemy), With<Enemy>>,
    enemy_config: Res<EnemyConfig>,
) {
    let mut combinations = enemy_query.iter_combinations_mut();
    while let Some(
        [
            (_entity_a, mut transform_a, enemy_a),
            (_entity_b, mut transform_b, enemy_b),
        ],
    ) = combinations.fetch_next()
    {
        if enemy_a.is_dying || enemy_b.is_dying {
            continue; // Skip dying enemies
        }

        if let Some((push_a, push_b)) = calculate_pushback(
            transform_a.translation,
            transform_b.translation,
            enemy_config.collision_distance,
        ) {
            transform_a.translation += push_a;
            transform_b.translation += push_b;

            // Keep enemies on ground
            transform_a.translation.y = 1.0;
            transform_b.translation.y = 1.0;
        }
    }
}
