use bevy::prelude::*;
use crate::components::AreaEffectType;
use serde::{Deserialize, Serialize};
use validator::Validate;


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

#[derive(Resource, Serialize, Deserialize, Clone, Debug)]
pub struct GameConfig {
    pub username: String,
    pub score: u32,
    pub settings: GameSettings,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            username: String::new(),
            score: 0,
            settings: GameSettings::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Validate)]
// NOTE: When adding new fields, update the default config.toml example in the project root
pub struct GameSettings {
    // Player settings
    #[validate(range(min = 0.1, max = 50.0))]
    pub player_movement_speed: f32,
    #[validate(range(min = 1.0, max = 1000.0))]
    pub player_max_health: f32,
    #[validate(range(min = 1.0, max = 500.0))]
    pub player_max_mana: f32,
    #[validate(range(min = 1.0, max = 500.0))]
    pub player_max_energy: f32,
    
    // Combat settings
    #[validate(range(min = 1.0, max = 100.0))]
    pub bullet_speed: f32,
    #[validate(range(min = 0.1, max = 50.0))]
    pub bullet_damage: f32,
    #[validate(range(min = 0.5, max = 10.0))]
    pub bullet_lifetime: f32,
    #[validate(range(min = 1.0, max = 1000.0))]
    pub magic_damage_per_second: f32,
    #[validate(range(min = 1.0, max = 1000.0))]
    pub poison_damage_per_second: f32,
    #[validate(range(min = 0.5, max = 20.0))]
    pub magic_area_radius: f32,
    #[validate(range(min = 0.5, max = 20.0))]
    pub poison_area_radius: f32,
    #[validate(range(min = 0.1, max = 30.0))]
    pub magic_area_duration: f32,
    #[validate(range(min = 0.1, max = 30.0))]
    pub poison_area_duration: f32,
    
    // Enemy settings
    #[validate(range(min = 0.1, max = 20.0))]
    pub enemy_movement_speed: f32,
    #[validate(range(min = 0.1, max = 100.0))]
    pub enemy_max_health: f32,
    #[validate(range(min = 1.0, max = 200.0))]
    pub enemy_max_mana: f32,
    #[validate(range(min = 1.0, max = 200.0))]
    pub enemy_max_energy: f32,
    #[validate(range(min = 1.0, max = 50.0))]
    pub enemy_chase_distance: f32,
    #[validate(range(min = 0.1, max = 5.0))]
    pub enemy_collision_distance: f32,
    #[validate(range(min = 1.0, max = 20.0))]
    pub enemy_spawn_distance_min: f32,
    #[validate(range(min = 5.0, max = 50.0))]
    pub enemy_spawn_distance_max: f32,
    #[validate(range(min = 0.1, max = 5.0))]
    pub bullet_collision_distance: f32,
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
    #[validate(range(min = 0.1, max = 2.0))]
    pub player_stopping_distance: f32,
    #[validate(range(min = 0.5, max = 10.0))]
    pub player_slowdown_distance: f32,
    
    #[validate(range(min = 0.1, max = 5.0))]
    pub enemy_stopping_distance: f32,
    #[validate(range(min = 0.1, max = 1.0))]
    pub enemy_speed_multiplier: f32,
    
    // Global LOD settings (applies to all characters)
    pub max_lod_level: String,           // Maximum LOD level: "high", "medium", or "low"
    #[validate(range(min = 1.0, max = 20.0))]
    pub enemy_lod_distance_high: f32,    // Distance to switch to medium LOD  
    #[validate(range(min = 5.0, max = 50.0))]
    pub enemy_lod_distance_low: f32,     // Distance to switch to low LOD
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            // Player settings
            player_movement_speed: 5.0,
            player_max_health: 100.0,
            player_max_mana: 50.0,
            player_max_energy: 100.0,
            
            // Combat settings
            bullet_speed: 15.0,
            bullet_damage: 2.0,
            bullet_lifetime: 3.0,
            magic_damage_per_second: 150.0,
            poison_damage_per_second: 80.0,
            magic_area_radius: 3.0,
            poison_area_radius: 4.0,
            magic_area_duration: 2.0,
            poison_area_duration: 4.0,
            
            // Enemy settings
            enemy_movement_speed: 3.0,
            enemy_max_health: 3.0,
            enemy_max_mana: 25.0,
            enemy_max_energy: 50.0,
            enemy_chase_distance: 8.0,
            enemy_collision_distance: 1.2,
            enemy_spawn_distance_min: 5.0,
            enemy_spawn_distance_max: 10.5,
            bullet_collision_distance: 0.6,
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
            player_stopping_distance: 0.5,
            player_slowdown_distance: 2.0,
            
            enemy_stopping_distance: 1.5,
            enemy_speed_multiplier: 0.8,
            
            // Global LOD settings
            max_lod_level: "high".to_string(),    // Maximum detail level for all characters
            enemy_lod_distance_high: 5.0,    // Switch to medium LOD at 5 units
            enemy_lod_distance_low: 15.0,    // Switch to low LOD at 15 units
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