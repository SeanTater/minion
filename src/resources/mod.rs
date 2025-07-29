use bevy::prelude::*;

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
    pub area_effect_radius: f32,
    pub area_effect_dps: i32,
    pub area_effect_duration: f32,
    pub collision_distance: f32,
    pub enemy_collision_distance: f32,
}

impl Default for CombatConfig {
    fn default() -> Self {
        Self {
            bullet_speed: 15.0,
            bullet_lifetime: 3.0,
            bullet_damage: 2,
            area_effect_radius: 3.0,
            area_effect_dps: 100,
            area_effect_duration: 2.0,
            collision_distance: 0.6,
            enemy_collision_distance: 1.2,
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