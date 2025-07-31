use crate::{components::*, game_logic::*, resources::*};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RespawnCounter { count: 0 })
            .add_systems(OnEnter(GameState::Playing), spawn_enemies)
            .add_systems(Update, (enemy_ai, update_enemy_lod).run_if(in_state(GameState::Playing)))
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
            Vec3::new(5.0, 1.0, 5.0),
            Vec3::new(-5.0, 1.0, 5.0),
            Vec3::new(5.0, 1.0, -5.0),
            Vec3::new(-5.0, 1.0, -5.0),
            Vec3::new(0.0, 1.0, 8.0),
        ];

        // Load all LOD levels for enemies
        let high_scene = asset_server.load("enemies/dark-knight-high.glb#Scene0");
        let med_scene = asset_server.load("enemies/dark-knight-med.glb#Scene0");  
        let low_scene = asset_server.load("enemies/dark-knight-low.glb#Scene0");

        // Determine starting LOD level based on global max setting
        let (starting_scene, starting_level) = match game_config.settings.max_lod_level.as_str() {
            "medium" => (med_scene.clone(), LodLevel::Medium),
            "low" => (low_scene.clone(), LodLevel::Low),
            _ => (high_scene.clone(), LodLevel::High),
        };

        for pos in spawn_positions {
            commands.spawn((
                SceneRoot(starting_scene.clone()), // Start with appropriate max LOD
                Transform::from_translation(pos)
                    .with_scale(Vec3::splat(2.0))
                    .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
                RigidBody::Dynamic,
                Collider::capsule_y(1.0, 0.5), // 2m tall capsule like player
                LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z, // Prevent tipping over
                Friction::coefficient(0.7),
                Restitution::coefficient(0.0), // No bouncing
                ColliderMassProperties::Density(0.8), // Slightly lighter than player
                ExternalForce::default(),
                Velocity::default(),
                Damping { linear_damping: 3.0, angular_damping: 8.0 }, // Add damping for more realistic movement
                Enemy {
                    speed: Speed::new(game_config.settings.enemy_movement_speed),
                    health: HealthPool::new_full(game_config.settings.enemy_max_health),
                    mana: ManaPool::new_full(game_config.settings.enemy_max_mana),
                    energy: EnergyPool::new_full(game_config.settings.enemy_max_energy),
                    chase_distance: Distance::new(game_config.settings.enemy_chase_distance),
                    is_dying: false,
                },
                EnemyLod {
                    current_level: starting_level,
                    high_handle: high_scene.clone(),
                    med_handle: med_scene.clone(),
                    low_handle: low_scene.clone(),
                },
                Name(generate_dark_name()),
            ));
        }
    }
}

fn enemy_ai(
    mut enemy_query: Query<(&Transform, &Enemy, &mut ExternalForce, &mut Velocity), (With<Enemy>, Without<Player>)>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    game_config: Res<GameConfig>,
) {
    if let Ok(player_transform) = player_query.single() {
        let player_pos_2d = Vec3::new(player_transform.translation.x, 0.0, player_transform.translation.z);
        
        // First pass: collect all enemy positions for separation calculation
        let enemy_positions: Vec<(Vec3, bool)> = enemy_query.iter()
            .map(|(transform, enemy, _, _)| {
                let pos = Vec3::new(transform.translation.x, 0.0, transform.translation.z);
                (pos, enemy.is_dying)
            })
            .collect();
        
        // Second pass: update each enemy with separation forces
        for (i, (enemy_transform, enemy, mut ext_force, mut velocity)) in enemy_query.iter_mut().enumerate() {
            if enemy.is_dying {
                ext_force.force = Vec3::ZERO;
                ext_force.torque = Vec3::ZERO;
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
                
                // Velocity-based movement like player
                let max_speed = enemy.speed.0 * game_config.settings.enemy_speed_multiplier;
                let desired_velocity = direction * max_speed;
                let current_velocity = Vec3::new(velocity.linvel.x, 0.0, velocity.linvel.z);
                let velocity_diff = desired_velocity - current_velocity;
                
                // Apply configurable force for movement
                let acceleration_force = velocity_diff * game_config.settings.enemy_acceleration_force;
                ext_force.force = Vec3::new(acceleration_force.x, 0.0, acceleration_force.z);
                
                // Apply torque for rotation toward player - FIXED: add PI/2 to face forward correctly
                let target_yaw = direction.z.atan2(direction.x) + std::f32::consts::FRAC_PI_2;
                let current_yaw = enemy_transform.rotation.to_euler(EulerRot::YXZ).0;
                let mut yaw_diff = target_yaw - current_yaw;
                
                // Normalize rotation difference
                while yaw_diff > std::f32::consts::PI {
                    yaw_diff -= 2.0 * std::f32::consts::PI;
                }
                while yaw_diff < -std::f32::consts::PI {
                    yaw_diff += 2.0 * std::f32::consts::PI;
                }
                
                ext_force.torque = Vec3::new(0.0, yaw_diff * game_config.settings.enemy_rotation_torque, 0.0);
            } else {
                // Apply configurable braking when not chasing
                let current_velocity = Vec3::new(velocity.linvel.x, 0.0, velocity.linvel.z);
                ext_force.force = -current_velocity * game_config.settings.enemy_braking_force;
                ext_force.torque = Vec3::ZERO;
            }
        }
    }
}


fn update_enemy_lod(
    mut enemy_query: Query<(&Transform, &mut EnemyLod, &mut SceneRoot), With<Enemy>>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    game_config: Res<GameConfig>,
) {
    if let Ok(player_transform) = player_query.single() {
        let player_pos = Vec3::new(player_transform.translation.x, 0.0, player_transform.translation.z);
        
        for (enemy_transform, mut enemy_lod, mut scene_root) in enemy_query.iter_mut() {
            let enemy_pos = Vec3::new(enemy_transform.translation.x, 0.0, enemy_transform.translation.z);
            let distance = player_pos.distance(enemy_pos);
            
            // Determine required LOD level based on distance
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
            if enemy_lod.current_level != required_lod {
                let new_scene = match required_lod {
                    LodLevel::High => enemy_lod.high_handle.clone(),
                    LodLevel::Medium => enemy_lod.med_handle.clone(),
                    LodLevel::Low => enemy_lod.low_handle.clone(),
                };
                
                scene_root.0 = new_scene;
                enemy_lod.current_level = required_lod;
                
                println!("Enemy switched to {:?} LOD at distance {:.1}", required_lod, distance);
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
