//! Geometric collision shapes for obstacle detection

use crate::pathfinding::NavigationGrid;
use bevy::prelude::*;

/// Geometric shapes for collision detection
#[derive(Debug, Clone)]
pub enum CollisionShape {
    Circle { radius: f32 },
    Rectangle { half_extents: Vec3 },
    Capsule { radius: f32, height: f32 },
    Compound { shapes: Vec<(Vec3, CollisionShape)> },
    None,
}

impl CollisionShape {
    /// Check if a world position is inside this shape
    pub fn contains_point(&self, world_pos: Vec3, shape_center: Vec3) -> bool {
        match self {
            CollisionShape::Circle { radius } => {
                let distance_2d =
                    Vec2::new(world_pos.x - shape_center.x, world_pos.z - shape_center.z).length();
                distance_2d <= *radius
            }
            CollisionShape::Rectangle { half_extents } => {
                let rel_pos = world_pos - shape_center;
                rel_pos.x.abs() <= half_extents.x && rel_pos.z.abs() <= half_extents.z
            }
            CollisionShape::Capsule { radius, .. } => {
                // Treat as circle for 2D pathfinding
                let distance_2d =
                    Vec2::new(world_pos.x - shape_center.x, world_pos.z - shape_center.z).length();
                distance_2d <= *radius
            }
            CollisionShape::Compound { shapes } => shapes
                .iter()
                .any(|(offset, shape)| shape.contains_point(world_pos, shape_center + *offset)),
            CollisionShape::None => false,
        }
    }

    /// Apply this shape to the navigation grid
    pub fn block_navigation_grid(&self, nav_grid: &mut NavigationGrid, center: Vec3, priority: u8) {
        match self {
            CollisionShape::Circle { radius } => {
                crate::pathfinding::grid_blocking::block_circular_area_with_priority(
                    nav_grid, center, *radius, priority,
                );
            }
            CollisionShape::Rectangle { half_extents } => {
                crate::pathfinding::grid_blocking::block_rectangular_area_with_priority(
                    nav_grid,
                    center,
                    *half_extents,
                    priority,
                );
            }
            CollisionShape::Capsule { radius, .. } => {
                crate::pathfinding::grid_blocking::block_circular_area_with_priority(
                    nav_grid, center, *radius, priority,
                );
            }
            CollisionShape::Compound { shapes } => {
                for (offset, shape) in shapes {
                    shape.block_navigation_grid(nav_grid, center + *offset, priority);
                }
            }
            CollisionShape::None => {}
        }
    }

    /// Get approximate bounds for spatial optimization
    pub fn approximate_bounds(&self, center: Vec3) -> (Vec3, Vec3) {
        match self {
            CollisionShape::Circle { radius } => {
                let extent = Vec3::new(*radius, 0.0, *radius);
                (center - extent, center + extent)
            }
            CollisionShape::Rectangle { half_extents } => {
                (center - *half_extents, center + *half_extents)
            }
            CollisionShape::Capsule { radius, height } => {
                let extent = Vec3::new(*radius, *height * 0.5, *radius);
                (center - extent, center + extent)
            }
            CollisionShape::Compound { shapes } => {
                let mut min_bound = center;
                let mut max_bound = center;
                for (offset, shape) in shapes {
                    let (shape_min, shape_max) = shape.approximate_bounds(center + *offset);
                    min_bound = min_bound.min(shape_min);
                    max_bound = max_bound.max(shape_max);
                }
                (min_bound, max_bound)
            }
            CollisionShape::None => (center, center),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circle_contains_point() {
        let shape = CollisionShape::Circle { radius: 2.0 };
        let center = Vec3::new(0.0, 0.0, 0.0);

        assert!(shape.contains_point(Vec3::new(1.0, 0.0, 1.0), center));
        assert!(!shape.contains_point(Vec3::new(3.0, 0.0, 0.0), center));
    }

    #[test]
    fn test_rectangle_contains_point() {
        let shape = CollisionShape::Rectangle {
            half_extents: Vec3::new(2.0, 1.0, 2.0),
        };
        let center = Vec3::new(0.0, 0.0, 0.0);

        assert!(shape.contains_point(Vec3::new(1.5, 0.0, 1.5), center));
        assert!(!shape.contains_point(Vec3::new(2.5, 0.0, 0.0), center));
    }

    #[test]
    fn test_capsule_contains_point() {
        let shape = CollisionShape::Capsule {
            radius: 1.5,
            height: 3.0,
        };
        let center = Vec3::new(0.0, 0.0, 0.0);

        // Capsule treated as circle for 2D pathfinding
        assert!(shape.contains_point(Vec3::new(1.0, 0.0, 1.0), center));
        assert!(!shape.contains_point(Vec3::new(2.0, 0.0, 0.0), center));
    }

    #[test]
    fn test_compound_contains_point() {
        let shapes = vec![
            (
                Vec3::new(1.0, 0.0, 0.0),
                CollisionShape::Circle { radius: 0.5 },
            ),
            (
                Vec3::new(-1.0, 0.0, 0.0),
                CollisionShape::Circle { radius: 0.5 },
            ),
        ];
        let compound = CollisionShape::Compound { shapes };
        let center = Vec3::new(0.0, 0.0, 0.0);

        // Should be inside first circle
        assert!(compound.contains_point(Vec3::new(1.2, 0.0, 0.0), center));
        // Should be inside second circle
        assert!(compound.contains_point(Vec3::new(-1.2, 0.0, 0.0), center));
        // Should be outside both
        assert!(!compound.contains_point(Vec3::new(0.0, 0.0, 2.0), center));
    }

    #[test]
    fn test_none_contains_point() {
        let shape = CollisionShape::None;
        let center = Vec3::new(0.0, 0.0, 0.0);

        assert!(!shape.contains_point(Vec3::new(0.0, 0.0, 0.0), center));
        assert!(!shape.contains_point(Vec3::new(10.0, 0.0, 10.0), center));
    }

    #[test]
    fn test_approximate_bounds() {
        // Test circle bounds
        let circle = CollisionShape::Circle { radius: 2.0 };
        let center = Vec3::new(5.0, 0.0, 5.0);
        let (min, max) = circle.approximate_bounds(center);
        assert_eq!(min, Vec3::new(3.0, 0.0, 3.0));
        assert_eq!(max, Vec3::new(7.0, 0.0, 7.0));

        // Test rectangle bounds
        let rect = CollisionShape::Rectangle {
            half_extents: Vec3::new(1.0, 2.0, 1.5),
        };
        let (min, max) = rect.approximate_bounds(center);
        assert_eq!(min, Vec3::new(4.0, -2.0, 3.5));
        assert_eq!(max, Vec3::new(6.0, 2.0, 6.5));

        // Test none bounds
        let none = CollisionShape::None;
        let (min, max) = none.approximate_bounds(center);
        assert_eq!(min, center);
        assert_eq!(max, center);
    }
}
