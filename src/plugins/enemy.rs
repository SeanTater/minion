use crate::{components::*, resources::*};
use bevy::prelude::*;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RespawnCounter { count: 0 })
            .add_systems(OnEnter(GameState::Playing), spawn_enemies)
            .add_systems(Update, (enemy_ai, update_entity_lod).run_if(in_state(GameState::Playing)))
            .add_systems(OnExit(GameState::Playing), cleanup_enemies);
    }
}

fn spawn_enemies(
    mut commands: Commands,
    game_config: Res<GameConfig>,
    enemy_query: Query<&Enemy>,
    asset_server: Res<AssetServer>,
) {
    // Only spawn enemies if none exist
    if enemy_query.is_empty() {
        let spawn_positions = [
            Vec3::new(5.0, 2.0, 5.0),
            Vec3::new(-5.0, 2.0, 5.0),
            Vec3::new(5.0, 2.0, -5.0),
            Vec3::new(-5.0, 2.0, -5.0),
            Vec3::new(0.0, 2.0, 8.0),
        ];

        for pos in spawn_positions {
            crate::game_logic::spawning::spawn_enemy_entity(&mut commands, &asset_server, pos, &game_config);
        }
    }
}

fn enemy_ai(
    mut enemy_query: Query<(&mut Transform, &Enemy), (With<Enemy>, Without<Player>)>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    game_config: Res<GameConfig>,
    time: Res<Time>,
) {
    if let Ok(player_transform) = player_query.single() {
        let player_pos_2d = Vec3::new(player_transform.translation.x, 0.0, player_transform.translation.z);
        
        // First pass: collect all enemy positions for separation calculation
        let enemy_positions: Vec<(Vec3, bool)> = enemy_query.iter()
            .map(|(transform, enemy)| {
                let pos = Vec3::new(transform.translation.x, 0.0, transform.translation.z);
                (pos, enemy.is_dying)
            })
            .collect();
        
        // Second pass: update each enemy with kinematic movement
        for (i, (mut enemy_transform, enemy)) in enemy_query.iter_mut().enumerate() {
            if enemy.is_dying {
                continue; // Skip dying enemies
            }

            // Use 2D distance for movement (ignore Y differences)
            let enemy_pos_2d = Vec3::new(enemy_transform.translation.x, 0.0, enemy_transform.translation.z);
            let distance = enemy_pos_2d.distance(player_pos_2d);

            if distance <= enemy.chase_distance.0 && distance > game_config.settings.enemy_stopping_distance {
                let mut direction = (player_pos_2d - enemy_pos_2d).normalize();
                
                // Add separation force - avoid other enemies
                let mut separation_force = Vec3::ZERO;
                let separation_radius = 2.0; // How far to avoid other enemies
                
                // Check against all other enemies for separation
                for (j, (other_pos_2d, other_is_dying)) in enemy_positions.iter().enumerate() {
                    if i == j { continue; } // Skip self
                    if *other_is_dying { continue; } // Skip dying enemies
                    
                    let other_distance = enemy_pos_2d.distance(*other_pos_2d);
                    
                    if other_distance < separation_radius && other_distance > 0.1 {
                        let away_from_other = (enemy_pos_2d - *other_pos_2d).normalize();
                        let separation_strength = (separation_radius - other_distance) / separation_radius;
                        separation_force += away_from_other * separation_strength;
                    }
                }
                
                // Blend seek (toward player) with separation (away from other enemies)
                direction = (direction + separation_force * 0.5).normalize_or_zero();
                
                // Direct kinematic movement
                let max_speed = enemy.speed.0 * game_config.settings.enemy_speed_multiplier;
                let move_distance = max_speed * time.delta_secs();
                let movement = direction * move_distance;
                enemy_transform.translation += movement;
                
                // Rotate toward player
                // NOTE: GLB models are facing backwards, so we flip the direction
                if direction.length() > 0.1 {
                    let character_pos = enemy_transform.translation;
                    let flat_target = Vec3::new(
                        character_pos.x - direction.x, // Flip for GLB orientation
                        character_pos.y, // Keep same Y level
                        character_pos.z - direction.z  // Flip for GLB orientation
                    );
                    enemy_transform.look_at(flat_target, Vec3::Y);
                }
            }
        }
    }
}


fn update_entity_lod(
    mut lod_query: Query<(&Transform, &mut LodEntity, &mut SceneRoot)>,
    player_query: Query<&Transform, With<Player>>,
    game_config: Res<GameConfig>,
) {
    if let Ok(player_transform) = player_query.single() {
        let player_pos = Vec3::new(player_transform.translation.x, 0.0, player_transform.translation.z);
        
        for (entity_transform, mut lod_entity, mut scene_root) in lod_query.iter_mut() {
            let entity_pos = Vec3::new(entity_transform.translation.x, 0.0, entity_transform.translation.z);
            
            // Calculate distance - for player entities, use a fixed close distance since we're always looking at them
            let distance = match lod_entity.entity_type {
                LodEntityType::Player => 5.0, // Player is always "close" for third-person view
                LodEntityType::Enemy => player_pos.distance(entity_pos),
            };
            
            // Determine required LOD level based on distance (use enemy settings for both types)
            let desired_lod = if distance <= game_config.settings.enemy_lod_distance_high {
                LodLevel::High
            } else if distance <= game_config.settings.enemy_lod_distance_low {
                LodLevel::Medium
            } else {
                LodLevel::Low
            };
            
            // Apply global max LOD level cap
            let max_lod = match game_config.settings.max_lod_level.as_str() {
                "medium" => LodLevel::Medium,
                "low" => LodLevel::Low,
                _ => LodLevel::High, // Default to high if invalid string
            };
            
            let required_lod = match (desired_lod, max_lod) {
                (LodLevel::High, LodLevel::Medium) | (LodLevel::High, LodLevel::Low) => max_lod,
                (LodLevel::Medium, LodLevel::Low) => LodLevel::Low,
                _ => desired_lod,
            };
            
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
                println!("{} switched to {:?} LOD at distance {:.1}", entity_type_str, required_lod, distance);
            }
        }
    }
}

fn cleanup_enemies(
    mut commands: Commands,
    enemy_query: Query<Entity, With<Enemy>>,
) {
    for entity in enemy_query.iter() {
        commands.entity(entity).despawn();
    }
}
