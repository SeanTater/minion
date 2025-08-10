use crate::components::AreaEffectType;
use crate::config::range_types::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Resource)]
pub struct RespawnCounter {
    pub count: u32,
}

#[derive(Resource)]
pub struct SelectedAreaEffect {
    pub effect_type: AreaEffectType,
}

impl Default for SelectedAreaEffect {
    fn default() -> Self {
        Self {
            effect_type: AreaEffectType::Magic,
        }
    }
}

#[derive(Resource, Serialize, Deserialize, Clone, Debug, Default)]
pub struct GameConfig {
    pub username: String,
    pub score: u32,
    pub settings: GameSettings,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
// NOTE: When adding new fields, update the default config.toml example in the project root
pub struct GameSettings {
    // Player settings
    pub player_movement_speed: MovementSpeed,
    pub player_max_health: HealthValue,
    pub player_max_mana: ManaValue,
    pub player_max_energy: EnergyValue,

    // Combat settings
    pub bullet_speed: BulletSpeed,
    pub bullet_damage: BulletDamage,
    pub bullet_lifetime: Lifetime,
    pub magic_damage_per_second: DamagePerSecond,
    pub poison_damage_per_second: DamagePerSecond,
    pub magic_area_radius: AreaRadius,
    pub poison_area_radius: AreaRadius,
    pub magic_area_duration: AreaDuration,
    pub poison_area_duration: AreaDuration,

    // Enemy settings
    pub enemy_movement_speed: MovementSpeed,
    pub enemy_max_health: HealthValue,
    pub enemy_max_mana: ManaValue,
    pub enemy_max_energy: EnergyValue,
    pub enemy_chase_distance: ChaseDistance,
    pub enemy_collision_distance: CollisionDistance,
    pub enemy_spawn_distance_min: SpawnDistanceMin,
    pub enemy_spawn_distance_max: SpawnDistanceMax,
    pub bullet_collision_distance: CollisionDistance,
    pub score_per_enemy: u32,

    // UI settings
    pub window_width: f32,
    pub window_height: f32,
    pub hud_font_size: f32,
    pub tooltip_font_size: f32,
    pub max_username_length: usize,

    // Visual settings
    pub ambient_light_brightness: f32,
    pub health_bar_color: [f32; 3],
    pub mana_bar_color: [f32; 3],
    pub energy_bar_color: [f32; 3],

    // Movement settings
    pub player_stopping_distance: StoppingDistance,
    pub player_slowdown_distance: SlowdownDistance,

    pub enemy_stopping_distance: StoppingDistance,
    pub enemy_speed_multiplier: SpeedMultiplier,

    // Global LOD settings (applies to all characters)
    pub max_lod_level: String, // Maximum LOD level: "high", "medium", or "low"
    pub enemy_lod_distance_high: LodDistance, // Distance to switch to medium LOD
    pub enemy_lod_distance_low: LodDistance, // Distance to switch to low LOD

    // Map settings
    pub map_file_path: String, // Path to map file relative to maps directory
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            // Player settings
            player_movement_speed: MovementSpeed::new(5.0),
            player_max_health: HealthValue::new(100.0),
            player_max_mana: ManaValue::new(50.0),
            player_max_energy: EnergyValue::new(100.0),

            // Combat settings
            bullet_speed: BulletSpeed::new(15.0),
            bullet_damage: BulletDamage::new(2.0),
            bullet_lifetime: Lifetime::new(3.0),
            magic_damage_per_second: DamagePerSecond::new(150.0),
            poison_damage_per_second: DamagePerSecond::new(80.0),
            magic_area_radius: AreaRadius::new(3.0),
            poison_area_radius: AreaRadius::new(4.0),
            magic_area_duration: AreaDuration::new(2.0),
            poison_area_duration: AreaDuration::new(4.0),

            // Enemy settings
            enemy_movement_speed: MovementSpeed::new(3.0),
            enemy_max_health: HealthValue::new(3.0),
            enemy_max_mana: ManaValue::new(25.0),
            enemy_max_energy: EnergyValue::new(50.0),
            enemy_chase_distance: ChaseDistance::new(8.0),
            enemy_collision_distance: CollisionDistance::new(1.2),
            enemy_spawn_distance_min: SpawnDistanceMin::new(5.0),
            enemy_spawn_distance_max: SpawnDistanceMax::new(10.5),
            bullet_collision_distance: CollisionDistance::new(0.6),
            score_per_enemy: 10,

            // UI settings
            window_width: 1280.0,
            window_height: 720.0,
            hud_font_size: 16.0,
            tooltip_font_size: 11.0,
            max_username_length: 20,

            // Visual settings
            ambient_light_brightness: 300.0,
            health_bar_color: [0.8, 0.2, 0.2],
            mana_bar_color: [0.2, 0.2, 0.8],
            energy_bar_color: [0.8, 0.8, 0.2],

            // Movement settings
            player_stopping_distance: StoppingDistance::new(0.5),
            player_slowdown_distance: SlowdownDistance::new(2.0),

            enemy_stopping_distance: StoppingDistance::new(1.5),
            enemy_speed_multiplier: SpeedMultiplier::new(0.8),

            // Global LOD settings
            max_lod_level: "high".to_string(), // Maximum detail level for all characters
            enemy_lod_distance_high: LodDistance::new(5.0), // Switch to medium LOD at 5 units
            enemy_lod_distance_low: LodDistance::new(15.0), // Switch to low LOD at 15 units

            // Map settings
            map_file_path: "generated_map.bin".to_string(), // Default map file
        }
    }
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    MainMenu,
    Settings,
    Playing,
}
