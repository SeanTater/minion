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

#[derive(Component)]
pub struct AreaEffect {
    pub radius: f32,
    pub damage_per_second: i32,
    pub duration: f32,
    pub elapsed: f32,
}