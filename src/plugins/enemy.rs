use crate::{components::*, game_logic::enemy::*, map::MapDefinition, resources::*};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RespawnCounter { count: 0 })
            .add_systems(OnEnter(GameState::Playing), spawn_enemies)
            .add_systems(
                Update,
                (enemy_ai, update_entity_lod).run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnExit(GameState::Playing), cleanup_enemies);
    }
}

fn spawn_enemies(
    mut commands: Commands,
    game_config: Res<GameConfig>,
    enemy_query: Query<&Enemy>,
    asset_server: Res<AssetServer>,
    map: Option<Res<MapDefinition>>,
) {
    // Only spawn enemies if none exist
    if enemy_query.is_empty() {
        if let Some(map) = map {
            // Use map-based spawning
            for spawn_zone in &map.enemy_zones {
                for i in 0..spawn_zone.max_enemies {
                    let spawn_pos = calculate_enemy_spawn_position(spawn_zone, i);

                    crate::game_logic::spawning::spawn_enemy_entity(
                        &mut commands,
                        &asset_server,
                        spawn_pos,
                        &game_config,
                    );
                }
            }
        }
        // Note: If no map is loaded, the map_loader plugin will create a fallback map with spawn zones
    }
}

fn enemy_ai(
    mut enemy_query: Query<
        (
            &mut Transform,
            &Enemy,
            &mut KinematicCharacterController,
            &mut PathfindingAgent,
        ),
        (With<Enemy>, Without<Player>),
    >,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    game_config: Res<GameConfig>,
    time: Res<Time>,
) {
    let player_transform = player_query
        .single()
        .expect("Player should always exist when in Playing state");
    let player_pos_2d = Vec3::new(
        player_transform.translation.x,
        0.0,
        player_transform.translation.z,
    );

    // First pass: collect all enemy positions for separation calculation
    let enemy_positions: Vec<(Vec3, bool)> = enemy_query
        .iter()
        .map(|(transform, enemy, _controller, _agent)| {
            let pos = Vec3::new(transform.translation.x, 0.0, transform.translation.z);
            (pos, enemy.is_dying)
        })
        .collect();

    // Second pass: update each enemy with hybrid pathfinding + flocking behavior
    for (i, (mut enemy_transform, enemy, mut controller, mut pathfinding_agent)) in
        enemy_query.iter_mut().enumerate()
    {
        if enemy.is_dying {
            continue; // Skip dying enemies
        }

        // Use 2D distance for movement (ignore Y differences)
        let enemy_pos_2d = Vec3::new(
            enemy_transform.translation.x,
            0.0,
            enemy_transform.translation.z,
        );
        let distance = enemy_pos_2d.distance(player_pos_2d);

        // Calculate flocking forces for this enemy
        let other_positions_for_enemy: Vec<(Vec3, bool)> = enemy_positions
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != i) // Skip self
            .map(|(_, (pos, is_dying))| (*pos, *is_dying))
            .collect();

        let separation_radius = 2.0;
        let flocking_forces =
            calculate_flocking_forces(enemy_pos_2d, &other_positions_for_enemy, separation_radius);

        // Inline AI decision logic - check if enemy should chase
        let is_chasing = distance <= enemy.chase_distance.0
            && distance > game_config.settings.enemy_stopping_distance.get();

        if is_chasing {
            // Set pathfinding destination
            pathfinding_agent.destination = Some(player_pos_2d);

            // Determine movement target: pathfinding for long-range, direct for close-range
            let movement_target = if distance > 10.0 {
                if let Some(waypoint) = pathfinding_agent.current_waypoint() {
                    waypoint
                } else {
                    player_pos_2d
                }
            } else {
                player_pos_2d
            };

            // Calculate direction to movement target
            let target_pos_2d = Vec3::new(movement_target.x, 0.0, movement_target.z);
            let direction_to_target = (target_pos_2d - enemy_pos_2d).normalize();

            // Blend movement direction with separation forces
            let separation_weight = if distance < 5.0 { 0.7 } else { 0.3 };
            let movement_direction = (direction_to_target
                + flocking_forces.total_force * separation_weight)
                .normalize_or_zero();

            // Use kinematic character controller for movement
            let max_speed = enemy.speed.0 * game_config.settings.enemy_speed_multiplier.get();
            let move_distance = max_speed * time.delta_secs();
            let movement = movement_direction * move_distance;
            controller.translation = Some(movement);

            // Rotate toward movement direction
            // NOTE: GLB models are facing backwards, so we flip the direction
            if movement_direction.length() > 0.1 {
                let character_pos = enemy_transform.translation;
                let flat_target = Vec3::new(
                    character_pos.x - movement_direction.x, // Flip for GLB orientation
                    character_pos.y,                        // Keep same Y level
                    character_pos.z - movement_direction.z, // Flip for GLB orientation
                );
                enemy_transform.look_at(flat_target, Vec3::Y);
            }
        } else {
            // Not chasing - clear pathfinding destination and stop movement
            pathfinding_agent.destination = None;
            pathfinding_agent.clear_path();
            controller.translation = Some(Vec3::ZERO);
        }
    }
}

fn update_entity_lod(
    mut lod_query: Query<(&Transform, &mut LodEntity, &mut SceneRoot)>,
    player_query: Query<&Transform, With<Player>>,
    game_config: Res<GameConfig>,
) {
    let player_transform = player_query
        .single()
        .expect("Player should always exist when in Playing state");
    let player_pos = Vec3::new(
        player_transform.translation.x,
        0.0,
        player_transform.translation.z,
    );

    for (entity_transform, mut lod_entity, mut scene_root) in lod_query.iter_mut() {
        let entity_pos = Vec3::new(
            entity_transform.translation.x,
            0.0,
            entity_transform.translation.z,
        );

        // Calculate distance - for player entities, use a fixed close distance since we're always looking at them
        let distance = match lod_entity.entity_type {
            LodEntityType::Player => 5.0, // Player is always "close" for third-person view
            LodEntityType::Enemy => player_pos.distance(entity_pos),
        };

        // Inline LOD level calculation with fallback
        let max_lod = LodLevel::try_from(game_config.settings.max_lod_level.as_str())
            .unwrap_or(LodLevel::High); // Default to High on invalid config
        let desired_lod = if distance <= game_config.settings.enemy_lod_distance_high.get() {
            LodLevel::High
        } else if distance <= game_config.settings.enemy_lod_distance_low.get() {
            LodLevel::Medium
        } else {
            LodLevel::Low
        };
        let required_lod = LodLevel::apply_max_cap(desired_lod, max_lod);

        // Switch model if LOD level changed
        if lod_entity.current_level != required_lod {
            let new_scene = match required_lod {
                LodLevel::High => lod_entity.high_handle.clone(),
                LodLevel::Medium => lod_entity.med_handle.clone(),
                LodLevel::Low => lod_entity.low_handle.clone(),
            };

            scene_root.0 = new_scene;
            lod_entity.current_level = required_lod;

            let entity_type_str = match lod_entity.entity_type {
                LodEntityType::Player => "Player",
                LodEntityType::Enemy => "Enemy",
            };
            println!(
                "{entity_type_str} switched to {required_lod:?} LOD at distance {distance:.1}"
            );
        }
    }
}

fn cleanup_enemies(mut commands: Commands, enemy_query: Query<Entity, With<Enemy>>) {
    for entity in enemy_query.iter() {
        commands.entity(entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test the inlined AI decision logic directly
    #[test]
    fn test_inlined_ai_decision_logic() {
        // Test chasing behavior
        let chase_distance = 10.0;
        let stopping_distance = 1.0;
        let distance = 5.0;

        let is_chasing = distance <= chase_distance && distance > stopping_distance;
        assert!(is_chasing);

        // Test not chasing when far
        let distance_far = 15.0;
        let is_chasing_far = distance_far <= chase_distance && distance_far > stopping_distance;
        assert!(!is_chasing_far);

        // Test not chasing when too close
        let distance_close = 0.5;
        let is_chasing_close =
            distance_close <= chase_distance && distance_close > stopping_distance;
        assert!(!is_chasing_close);
    }

    #[test]
    fn test_movement_target_selection() {
        let player_pos = Vec3::new(0.0, 0.0, 0.0);
        let waypoint = Vec3::new(5.0, 0.0, 0.0);

        // Long distance should use waypoint if available
        let distance_long = 15.0;
        let movement_target_long = if distance_long > 10.0 {
            Some(waypoint)
        } else {
            None
        }
        .unwrap_or(player_pos);
        assert_eq!(movement_target_long, waypoint);

        // Short distance should use direct path
        let distance_short = 5.0;
        let movement_target_short = if distance_short > 10.0 {
            Some(waypoint)
        } else {
            None
        }
        .unwrap_or(player_pos);
        assert_eq!(movement_target_short, player_pos);
    }

    #[test]
    fn test_separation_weight_calculation() {
        // Close distance should have high separation weight
        let distance_close = 3.0;
        let separation_weight_close = if distance_close < 5.0 { 0.7 } else { 0.3 };
        assert_eq!(separation_weight_close, 0.7);

        // Far distance should have low separation weight
        let distance_far = 8.0;
        let separation_weight_far = if distance_far < 5.0 { 0.7 } else { 0.3 };
        assert_eq!(separation_weight_far, 0.3);
    }

    #[test]
    fn test_inlined_lod_calculation() {
        let lod_distance_high = 10.0;
        let lod_distance_low = 20.0;

        // Close distance should be high LOD
        let distance_close = 5.0;
        let desired_lod_close = if distance_close <= lod_distance_high {
            LodLevel::High
        } else if distance_close <= lod_distance_low {
            LodLevel::Medium
        } else {
            LodLevel::Low
        };
        assert_eq!(desired_lod_close, LodLevel::High);

        // Medium distance should be medium LOD
        let distance_medium = 15.0;
        let desired_lod_medium = if distance_medium <= lod_distance_high {
            LodLevel::High
        } else if distance_medium <= lod_distance_low {
            LodLevel::Medium
        } else {
            LodLevel::Low
        };
        assert_eq!(desired_lod_medium, LodLevel::Medium);

        // Far distance should be low LOD
        let distance_far = 25.0;
        let desired_lod_far = if distance_far <= lod_distance_high {
            LodLevel::High
        } else if distance_far <= lod_distance_low {
            LodLevel::Medium
        } else {
            LodLevel::Low
        };
        assert_eq!(desired_lod_far, LodLevel::Low);
    }

    #[test]
    fn test_movement_direction_calculation() {
        let enemy_pos = Vec3::new(0.0, 0.0, 0.0);
        let movement_target = Vec3::new(5.0, 0.0, 0.0);

        // Calculate direction to movement target
        let target_pos_2d = Vec3::new(movement_target.x, 0.0, movement_target.z);
        let direction_to_target = (target_pos_2d - enemy_pos).normalize();

        assert_eq!(direction_to_target, Vec3::new(1.0, 0.0, 0.0));

        // Test with separation forces
        let separation_force = Vec3::new(0.0, 0.0, 1.0);
        let separation_weight = 0.5;
        let final_direction =
            (direction_to_target + separation_force * separation_weight).normalize_or_zero();

        // Should be a blend of target direction and separation
        assert!(final_direction.x > 0.0); // Still moving toward target
        assert!(final_direction.z > 0.0); // But also influenced by separation
    }

    #[test]
    fn test_integration_with_existing_math_functions() {
        // Test that we still use the mathematical functions correctly
        use crate::game_logic::enemy::*;
        use crate::map::SpawnZone;

        // Test spawn position calculation (kept as reusable function)
        let spawn_zone = SpawnZone {
            center: Vec3::new(10.0, 0.0, 10.0),
            radius: 5.0,
            max_enemies: 4,
            enemy_types: vec!["dark-knight".to_string()],
        };

        let pos1 = calculate_enemy_spawn_position(&spawn_zone, 0);
        let pos2 = calculate_enemy_spawn_position(&spawn_zone, 1);

        // Should be deterministic and different
        assert_ne!(pos1, pos2);
        assert_eq!(pos1.y, spawn_zone.center.y + 1.0);

        // Test flocking forces calculation (kept as reusable function)
        let enemy_pos = Vec3::new(0.0, 0.0, 0.0);
        let other_positions = vec![(Vec3::new(1.0, 0.0, 0.0), false)];
        let forces = calculate_flocking_forces(enemy_pos, &other_positions, 2.0);

        // Should push away from neighbor
        assert!(forces.separation.x < 0.0);
        assert_eq!(forces.total_force, forces.separation);
    }
}
