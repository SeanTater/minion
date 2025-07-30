use bevy::prelude::*;
use crate::components::{AreaEffectType, Speed, Distance, Damage};
use serde::{Deserialize, Serialize};


#[derive(Resource)]
pub struct RespawnCounter {
    pub count: u32,
}

#[derive(Resource, Debug, Clone)]
pub struct CombatConfig {
    pub bullet_speed: Speed,
    pub bullet_lifetime: f32,
    pub bullet_damage: Damage,
    pub collision_distance: Distance,
}

impl Default for CombatConfig {
    fn default() -> Self {
        Self {
            bullet_speed: Speed::new(15.0),
            bullet_lifetime: 3.0,
            bullet_damage: Damage::new(2.0),
            collision_distance: Distance::new(0.6),
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct EnemyConfig {
    pub speed: Speed,  
    pub max_health: f32,
    pub chase_distance: Distance,
    pub spawn_distance_min: Distance,
    pub spawn_distance_max: Distance,
    pub collision_distance: Distance,
}

impl Default for EnemyConfig {
    fn default() -> Self {
        Self {
            speed: Speed::new(3.0),
            max_health: 3.0,
            chase_distance: Distance::new(8.0),
            spawn_distance_min: Distance::new(5.0),
            spawn_distance_max: Distance::new(10.5),
            collision_distance: Distance::new(1.2),
        }
    }
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameSettings {
    // Player settings
    pub player_movement_speed: f32,
    pub player_max_health: f32,
    pub player_max_mana: f32,
    pub player_max_energy: f32,
    
    // Combat settings
    pub bullet_speed: f32,
    pub bullet_damage: f32,
    pub bullet_lifetime: f32,
    pub magic_damage_per_second: f32,
    pub poison_damage_per_second: f32,
    pub magic_area_radius: f32,
    pub poison_area_radius: f32,
    
    // Enemy settings
    pub enemy_movement_speed: f32,
    pub enemy_max_health: f32,
    pub enemy_chase_distance: f32,
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
            
            // Enemy settings
            enemy_movement_speed: 3.0,
            enemy_max_health: 3.0,
            enemy_chase_distance: 8.0,
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