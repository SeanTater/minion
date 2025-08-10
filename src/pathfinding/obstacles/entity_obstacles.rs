//! Dynamic entity obstacles for moving objects

use crate::pathfinding::obstacles::{CollisionShape, Obstacle};
use bevy::prelude::*;

/// Dynamic entity obstacle for moving objects
#[derive(Debug, Clone)]
pub struct EntityObstacle {
    pub entity_id: Entity,
    pub position: Vec3,
    pub collision_radius: f32,
    pub obstacle_type: EntityObstacleType,
}

#[derive(Debug, Clone)]
pub enum EntityObstacleType {
    Player,
    Enemy,
    Projectile,
    TemporaryEffect,
}

impl EntityObstacle {
    pub fn new(
        entity_id: Entity,
        position: Vec3,
        collision_radius: f32,
        obstacle_type: EntityObstacleType,
    ) -> Self {
        Self {
            entity_id,
            position,
            collision_radius,
            obstacle_type,
        }
    }

    /// Create player obstacle
    pub fn player(entity_id: Entity, position: Vec3, radius: f32) -> Self {
        Self::new(entity_id, position, radius, EntityObstacleType::Player)
    }

    /// Create enemy obstacle
    pub fn enemy(entity_id: Entity, position: Vec3, radius: f32) -> Self {
        Self::new(entity_id, position, radius, EntityObstacleType::Enemy)
    }

    /// Create projectile obstacle
    pub fn projectile(entity_id: Entity, position: Vec3, radius: f32) -> Self {
        Self::new(entity_id, position, radius, EntityObstacleType::Projectile)
    }

    /// Create temporary effect obstacle
    pub fn temporary_effect(entity_id: Entity, position: Vec3, radius: f32) -> Self {
        Self::new(
            entity_id,
            position,
            radius,
            EntityObstacleType::TemporaryEffect,
        )
    }
}

impl Obstacle for EntityObstacle {
    fn collision_shape(&self) -> CollisionShape {
        CollisionShape::Circle {
            radius: self.collision_radius,
        }
    }

    fn world_position(&self) -> Vec3 {
        self.position
    }

    fn blocking_priority(&self) -> u8 {
        match self.obstacle_type {
            EntityObstacleType::Player => 180,
            EntityObstacleType::Enemy => 160,
            EntityObstacleType::Projectile => 50,
            EntityObstacleType::TemporaryEffect => 80,
        }
    }
}

/// Component to mark entities as pathfinding obstacles
#[derive(Component, Debug, Clone)]
pub struct ObstacleSource {
    pub collision_radius: f32,
    pub obstacle_type: EntityObstacleType,
    pub blocks_pathfinding: bool,
}

impl ObstacleSource {
    pub fn new(collision_radius: f32, obstacle_type: EntityObstacleType) -> Self {
        Self {
            collision_radius,
            obstacle_type,
            blocks_pathfinding: true,
        }
    }

    pub fn player(collision_radius: f32) -> Self {
        Self::new(collision_radius, EntityObstacleType::Player)
    }

    pub fn enemy(collision_radius: f32) -> Self {
        Self::new(collision_radius, EntityObstacleType::Enemy)
    }

    pub fn projectile(collision_radius: f32) -> Self {
        Self::new(collision_radius, EntityObstacleType::Projectile)
    }

    pub fn temporary_effect(collision_radius: f32) -> Self {
        Self::new(collision_radius, EntityObstacleType::TemporaryEffect)
    }

    /// Disable pathfinding blocking for this obstacle source
    pub fn disable_blocking(&mut self) {
        self.blocks_pathfinding = false;
    }

    /// Enable pathfinding blocking for this obstacle source
    pub fn enable_blocking(&mut self) {
        self.blocks_pathfinding = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_obstacle_creation() {
        let entity = Entity::from_raw(123);
        let obstacle = EntityObstacle::player(entity, Vec3::new(5.0, 0.0, 5.0), 0.5);

        assert_eq!(obstacle.entity_id, entity);
        assert_eq!(obstacle.position, Vec3::new(5.0, 0.0, 5.0));
        assert_eq!(obstacle.collision_radius, 0.5);
        assert!(matches!(obstacle.obstacle_type, EntityObstacleType::Player));
        assert_eq!(obstacle.blocking_priority(), 180);
    }

    #[test]
    fn test_entity_obstacle_types() {
        let entity = Entity::from_raw(456);

        let player = EntityObstacle::player(entity, Vec3::ZERO, 0.5);
        let enemy = EntityObstacle::enemy(entity, Vec3::ZERO, 0.8);
        let projectile = EntityObstacle::projectile(entity, Vec3::ZERO, 0.1);
        let effect = EntityObstacle::temporary_effect(entity, Vec3::ZERO, 1.0);

        assert_eq!(player.blocking_priority(), 180);
        assert_eq!(enemy.blocking_priority(), 160);
        assert_eq!(projectile.blocking_priority(), 50);
        assert_eq!(effect.blocking_priority(), 80);
    }

    #[test]
    fn test_entity_obstacle_collision_shape() {
        let entity = Entity::from_raw(789);
        let obstacle = EntityObstacle::enemy(entity, Vec3::new(3.0, 0.0, 3.0), 1.2);

        match obstacle.collision_shape() {
            CollisionShape::Circle { radius } => {
                assert_eq!(radius, 1.2);
            }
            _ => panic!("Expected circle collision shape for entity obstacle"),
        }
    }

    #[test]
    fn test_entity_obstacle_contains_point() {
        let entity = Entity::from_raw(101);
        let obstacle = EntityObstacle::player(entity, Vec3::new(0.0, 0.0, 0.0), 1.0);

        // Point inside collision radius
        assert!(obstacle.contains_point(Vec3::new(0.8, 0.0, 0.0)));

        // Point outside collision radius
        assert!(!obstacle.contains_point(Vec3::new(1.5, 0.0, 0.0)));

        // Point exactly on boundary (should be inside)
        assert!(obstacle.contains_point(Vec3::new(1.0, 0.0, 0.0)));
    }

    #[test]
    fn test_obstacle_source_component() {
        let source = ObstacleSource::enemy(0.8);

        assert_eq!(source.collision_radius, 0.8);
        assert!(matches!(source.obstacle_type, EntityObstacleType::Enemy));
        assert!(source.blocks_pathfinding);
    }

    #[test]
    fn test_obstacle_source_blocking_control() {
        let mut source = ObstacleSource::player(0.5);

        assert!(source.blocks_pathfinding);

        source.disable_blocking();
        assert!(!source.blocks_pathfinding);

        source.enable_blocking();
        assert!(source.blocks_pathfinding);
    }

    #[test]
    fn test_obstacle_source_factory_methods() {
        let player_source = ObstacleSource::player(0.5);
        let enemy_source = ObstacleSource::enemy(0.8);
        let projectile_source = ObstacleSource::projectile(0.1);
        let effect_source = ObstacleSource::temporary_effect(1.5);

        assert!(matches!(
            player_source.obstacle_type,
            EntityObstacleType::Player
        ));
        assert!(matches!(
            enemy_source.obstacle_type,
            EntityObstacleType::Enemy
        ));
        assert!(matches!(
            projectile_source.obstacle_type,
            EntityObstacleType::Projectile
        ));
        assert!(matches!(
            effect_source.obstacle_type,
            EntityObstacleType::TemporaryEffect
        ));

        assert_eq!(player_source.collision_radius, 0.5);
        assert_eq!(enemy_source.collision_radius, 0.8);
        assert_eq!(projectile_source.collision_radius, 0.1);
        assert_eq!(effect_source.collision_radius, 1.5);
    }

    #[test]
    fn test_priority_ordering() {
        let entity = Entity::from_raw(999);

        let player = EntityObstacle::player(entity, Vec3::ZERO, 0.5);
        let enemy = EntityObstacle::enemy(entity, Vec3::ZERO, 0.5);
        let projectile = EntityObstacle::projectile(entity, Vec3::ZERO, 0.5);
        let effect = EntityObstacle::temporary_effect(entity, Vec3::ZERO, 0.5);

        // Priority ordering: projectile < effect < enemy < player
        assert!(projectile.blocking_priority() < effect.blocking_priority());
        assert!(effect.blocking_priority() < enemy.blocking_priority());
        assert!(enemy.blocking_priority() < player.blocking_priority());
    }
}
