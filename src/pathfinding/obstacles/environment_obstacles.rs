//! Static environment obstacles from map data

use crate::map::EnvironmentObject;
use crate::pathfinding::obstacles::{CollisionShape, Obstacle};
use bevy::prelude::*;

/// Static environment obstacle from map data
#[derive(Debug, Clone)]
pub struct EnvironmentObstacle {
    pub object_type: EnvironmentObjectType,
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
}

/// Strongly-typed environment object types
#[derive(Debug, Clone)]
pub enum EnvironmentObjectType {
    Tree {
        trunk_radius_factor: f32,
    },
    Rock {
        collision_factor: f32,
    },
    Boulder {
        collision_factor: f32,
    },
    Grass,
    Structure {
        collision_shape: CollisionShape,
    },
    Custom {
        name: String,
        collision_shape: CollisionShape,
    },
}

impl From<&EnvironmentObject> for EnvironmentObstacle {
    fn from(obj: &EnvironmentObject) -> Self {
        let object_type = match obj.object_type.as_str() {
            "tree" => EnvironmentObjectType::Tree {
                trunk_radius_factor: 0.3,
            },
            "rock" => EnvironmentObjectType::Rock {
                collision_factor: 0.5,
            },
            "boulder" => EnvironmentObjectType::Boulder {
                collision_factor: 0.5,
            },
            "grass" => EnvironmentObjectType::Grass,
            name => {
                // Default fallback for unknown types
                let collision_shape = if obj.scale.x > 0.0 && obj.scale.z > 0.0 {
                    CollisionShape::Rectangle {
                        half_extents: obj.scale * 0.5,
                    }
                } else {
                    CollisionShape::None
                };
                EnvironmentObjectType::Custom {
                    name: name.to_string(),
                    collision_shape,
                }
            }
        };

        Self {
            object_type,
            position: obj.position,
            rotation: obj.rotation,
            scale: obj.scale,
        }
    }
}

impl Obstacle for EnvironmentObstacle {
    fn collision_shape(&self) -> CollisionShape {
        let shape = match &self.object_type {
            EnvironmentObjectType::Tree {
                trunk_radius_factor,
            } => CollisionShape::Circle {
                // Increase tree collision radius for pathfinding to account for canopy
                radius: self.scale.x * trunk_radius_factor * 2.0, // 2x larger for pathfinding
            },
            EnvironmentObjectType::Rock { collision_factor }
            | EnvironmentObjectType::Boulder { collision_factor } => CollisionShape::Circle {
                // Increase rock/boulder collision radius for pathfinding safety margin
                radius: self.scale.x * collision_factor * 1.5, // 1.5x larger for pathfinding
            },
            EnvironmentObjectType::Grass => CollisionShape::None,
            EnvironmentObjectType::Structure { collision_shape } => collision_shape.clone(),
            EnvironmentObjectType::Custom {
                collision_shape, ..
            } => collision_shape.clone(),
        };

        // Debug logging for pathfinding obstacles
        if let CollisionShape::Circle { radius } = &shape {
            debug!(
                "Pathfinding obstacle: {} at ({:.1}, {:.1}, {:.1}) radius={:.1}",
                match &self.object_type {
                    EnvironmentObjectType::Tree { .. } => "tree",
                    EnvironmentObjectType::Rock { .. } => "rock",
                    EnvironmentObjectType::Boulder { .. } => "boulder",
                    EnvironmentObjectType::Custom { name, .. } => name,
                    _ => "other",
                },
                self.position.x,
                self.position.y,
                self.position.z,
                radius
            );
        }

        shape
    }

    fn blocks_pathfinding(&self) -> bool {
        match &self.object_type {
            EnvironmentObjectType::Grass => false,
            EnvironmentObjectType::Custom {
                collision_shape, ..
            } => !matches!(collision_shape, CollisionShape::None),
            _ => true,
        }
    }

    fn world_position(&self) -> Vec3 {
        self.position
    }

    fn blocking_priority(&self) -> u8 {
        match &self.object_type {
            EnvironmentObjectType::Tree { .. } => 150, // Trees are important obstacles
            EnvironmentObjectType::Boulder { .. } => 200, // Boulders are very solid
            EnvironmentObjectType::Rock { .. } => 120, // Rocks are medium priority
            EnvironmentObjectType::Structure { .. } => 255, // Structures are absolute
            EnvironmentObjectType::Custom { .. } => 100, // Default priority
            EnvironmentObjectType::Grass => 0,         // Grass doesn't block
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_obstacle_conversion() {
        let env_obj = EnvironmentObject::new(
            "tree".to_string(),
            Vec3::new(5.0, 0.0, 5.0),
            Vec3::ZERO,
            Vec3::new(2.0, 3.0, 2.0),
        );

        let obstacle = EnvironmentObstacle::from(&env_obj);
        assert!(matches!(
            obstacle.object_type,
            EnvironmentObjectType::Tree { .. }
        ));
        assert!(obstacle.blocks_pathfinding());
        assert_eq!(obstacle.blocking_priority(), 150);

        // Check collision shape
        match obstacle.collision_shape() {
            CollisionShape::Circle { radius } => {
                assert_eq!(radius, 2.0 * 0.3 * 2.0); // scale.x * trunk_radius_factor * 2.0 (pathfinding multiplier)
            }
            _ => panic!("Expected circle collision shape for tree"),
        }
    }

    #[test]
    fn test_rock_obstacle_conversion() {
        let env_obj = EnvironmentObject::new(
            "rock".to_string(),
            Vec3::new(3.0, 0.0, 3.0),
            Vec3::ZERO,
            Vec3::new(1.0, 1.0, 1.0),
        );

        let obstacle = EnvironmentObstacle::from(&env_obj);
        assert!(matches!(
            obstacle.object_type,
            EnvironmentObjectType::Rock { .. }
        ));
        assert!(obstacle.blocks_pathfinding());
        assert_eq!(obstacle.blocking_priority(), 120);

        match obstacle.collision_shape() {
            CollisionShape::Circle { radius } => {
                assert_eq!(radius, 1.0 * 0.5 * 1.5); // scale.x * collision_factor * 1.5 (pathfinding multiplier)
            }
            _ => panic!("Expected circle collision shape for rock"),
        }
    }

    #[test]
    fn test_boulder_obstacle_conversion() {
        let env_obj = EnvironmentObject::new(
            "boulder".to_string(),
            Vec3::new(1.0, 0.0, 1.0),
            Vec3::ZERO,
            Vec3::new(1.5, 1.5, 1.5),
        );

        let obstacle = EnvironmentObstacle::from(&env_obj);
        assert!(matches!(
            obstacle.object_type,
            EnvironmentObjectType::Boulder { .. }
        ));
        assert!(obstacle.blocks_pathfinding());
        assert_eq!(obstacle.blocking_priority(), 200);

        match obstacle.collision_shape() {
            CollisionShape::Circle { radius } => {
                assert_eq!(radius, 1.5 * 0.5 * 1.5); // scale.x * collision_factor * 1.5 (pathfinding multiplier)
            }
            _ => panic!("Expected circle collision shape for boulder"),
        }
    }

    #[test]
    fn test_grass_obstacle_no_blocking() {
        let env_obj = EnvironmentObject::new(
            "grass".to_string(),
            Vec3::new(1.0, 0.0, 1.0),
            Vec3::ZERO,
            Vec3::ONE,
        );

        let obstacle = EnvironmentObstacle::from(&env_obj);
        assert!(!obstacle.blocks_pathfinding());
        assert!(matches!(obstacle.collision_shape(), CollisionShape::None));
        assert_eq!(obstacle.blocking_priority(), 0);
    }

    #[test]
    fn test_custom_obstacle_fallback() {
        let env_obj = EnvironmentObject::new(
            "custom_building".to_string(),
            Vec3::new(10.0, 0.0, 10.0),
            Vec3::ZERO,
            Vec3::new(4.0, 6.0, 4.0),
        );

        let obstacle = EnvironmentObstacle::from(&env_obj);
        assert!(matches!(
            obstacle.object_type,
            EnvironmentObjectType::Custom { .. }
        ));
        assert!(obstacle.blocks_pathfinding());
        assert_eq!(obstacle.blocking_priority(), 100);

        match obstacle.collision_shape() {
            CollisionShape::Rectangle { half_extents } => {
                assert_eq!(half_extents, Vec3::new(2.0, 3.0, 2.0)); // scale * 0.5
            }
            _ => panic!("Expected rectangle collision shape for custom object"),
        }
    }

    #[test]
    fn test_invalid_scale_custom_obstacle() {
        let env_obj = EnvironmentObject::new(
            "invalid_object".to_string(),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::ZERO,
            Vec3::new(0.0, 1.0, 0.0), // Invalid scale (x=0, z=0)
        );

        let obstacle = EnvironmentObstacle::from(&env_obj);
        assert!(matches!(
            obstacle.object_type,
            EnvironmentObjectType::Custom { .. }
        ));
        assert!(matches!(obstacle.collision_shape(), CollisionShape::None));
        assert!(!obstacle.blocks_pathfinding()); // Should not block with None collision shape
    }

    #[test]
    fn test_obstacle_priority_ordering() {
        // Test that priorities are ordered correctly for obstacle override
        let grass = EnvironmentObstacle::from(&EnvironmentObject::new(
            "grass".to_string(),
            Vec3::ZERO,
            Vec3::ZERO,
            Vec3::ONE,
        ));

        let rock = EnvironmentObstacle::from(&EnvironmentObject::new(
            "rock".to_string(),
            Vec3::ZERO,
            Vec3::ZERO,
            Vec3::ONE,
        ));

        let tree = EnvironmentObstacle::from(&EnvironmentObject::new(
            "tree".to_string(),
            Vec3::ZERO,
            Vec3::ZERO,
            Vec3::ONE,
        ));

        let boulder = EnvironmentObstacle::from(&EnvironmentObject::new(
            "boulder".to_string(),
            Vec3::ZERO,
            Vec3::ZERO,
            Vec3::ONE,
        ));

        // Priority ordering: grass < rock < tree < boulder
        assert!(grass.blocking_priority() < rock.blocking_priority());
        assert!(rock.blocking_priority() < tree.blocking_priority());
        assert!(tree.blocking_priority() < boulder.blocking_priority());
    }

    #[test]
    fn test_collision_point_detection() {
        let tree = EnvironmentObstacle::from(&EnvironmentObject::new(
            "tree".to_string(),
            Vec3::new(5.0, 0.0, 5.0),
            Vec3::ZERO,
            Vec3::new(2.0, 3.0, 2.0), // Radius will be 2.0 * 0.3 * 2.0 = 1.2
        ));

        // Point inside tree collision radius
        assert!(tree.contains_point(Vec3::new(5.5, 0.0, 5.0)));

        // Point outside tree collision radius (now needs to be further due to larger radius)
        assert!(!tree.contains_point(Vec3::new(7.0, 0.0, 5.0)));

        // Grass should never contain points
        let grass = EnvironmentObstacle::from(&EnvironmentObject::new(
            "grass".to_string(),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::ZERO,
            Vec3::ONE,
        ));
        assert!(!grass.contains_point(Vec3::new(0.0, 0.0, 0.0)));
    }
}
