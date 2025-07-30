use bevy::prelude::*;
use crate::components::AreaEffectType;
use serde::{Deserialize, Serialize};

#[derive(Resource)]
pub struct ObjectPool<T: Component> {
    pub available: Vec<Entity>,
    pub _phantom: std::marker::PhantomData<T>,
}

impl<T: Component> Default for ObjectPool<T> {
    fn default() -> Self {
        Self {
            available: Vec::new(),
            _phantom: std::marker::PhantomData,
        }
    }
}

#[derive(Resource)]
pub struct RespawnCounter {
    pub count: u32,
}

#[derive(Resource, Debug, Clone)]
pub struct CombatConfig {
    pub bullet_speed: f32,
    pub bullet_lifetime: f32,
    pub bullet_damage: i32,
    pub collision_distance: f32,
}

impl Default for CombatConfig {
    fn default() -> Self {
        Self {
            bullet_speed: 15.0,
            bullet_lifetime: 3.0,
            bullet_damage: 2,
            collision_distance: 0.6,
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct EnemyConfig {
    pub speed: f32,
    pub health: i32,
    pub chase_distance: f32,
    pub spawn_distance_min: f32,
    pub spawn_distance_max: f32,
    pub collision_distance: f32,
}

impl Default for EnemyConfig {
    fn default() -> Self {
        Self {
            speed: 3.0,
            health: 3,
            chase_distance: 8.0,
            spawn_distance_min: 5.0,
            spawn_distance_max: 10.5,
            collision_distance: 1.2,
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