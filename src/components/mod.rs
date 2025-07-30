use bevy::prelude::*;

#[derive(Component)]
pub struct Player {
    pub move_target: Option<Vec3>,
    pub speed: f32,
}

#[derive(Component)]
pub struct Ground;

#[derive(Component)]
pub struct CameraFollow {
    pub offset: Vec3,
}

#[derive(Component)]
pub struct Enemy {
    pub speed: f32,
    pub health: i32,
    pub chase_distance: f32,
    pub is_dying: bool,
}

#[derive(Component)]
pub struct Bullet {
    pub direction: Vec3,
    pub speed: f32,
    pub lifetime: f32,
    pub damage: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AreaEffectType {
    Magic,
    Poison,
}

impl AreaEffectType {
    pub fn damage_per_second(&self) -> i32 {
        match self {
            AreaEffectType::Magic => 150,
            AreaEffectType::Poison => 80,
        }
    }
    
    pub fn radius(&self) -> f32 {
        match self {
            AreaEffectType::Magic => 3.0,
            AreaEffectType::Poison => 4.0,
        }
    }
    
    pub fn duration(&self) -> f32 {
        match self {
            AreaEffectType::Magic => 2.0,
            AreaEffectType::Poison => 4.0,
        }
    }
    
    pub fn base_color(&self) -> Color {
        match self {
            AreaEffectType::Magic => Color::srgba(0.0, 0.5, 1.0, 0.3),
            AreaEffectType::Poison => Color::srgba(0.0, 1.0, 0.2, 0.3),
        }
    }
}

#[derive(Component)]
pub struct AreaEffect {
    pub effect_type: AreaEffectType,
    pub elapsed: f32,
}