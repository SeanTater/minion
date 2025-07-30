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
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            username: String::new(),
            score: 0,
        }
    }
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    MainMenu,
    Playing,
}