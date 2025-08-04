//! Trait-based obstacle system for pathfinding collision detection

use crate::pathfinding::NavigationGrid;
use bevy::prelude::*;

pub mod collision_shapes;
pub mod entity_obstacles;
pub mod environment_obstacles;
pub mod obstacle_manager;

pub use collision_shapes::*;
pub use entity_obstacles::*;
pub use environment_obstacles::*;
pub use obstacle_manager::*;

/// Core trait for objects that can obstruct pathfinding
pub trait Obstacle: Send + Sync {
    /// Get the obstacle's collision shape for pathfinding
    fn collision_shape(&self) -> CollisionShape;

    /// Check if this obstacle blocks pathfinding at all
    fn blocks_pathfinding(&self) -> bool {
        true
    }

    /// Get obstacle priority for overlapping obstacles (higher = more important)
    fn blocking_priority(&self) -> u8 {
        100
    }

    /// Get the world position of this obstacle
    fn world_position(&self) -> Vec3;

    /// Test if a world position is inside this obstacle
    fn contains_point(&self, world_pos: Vec3) -> bool {
        self.collision_shape()
            .contains_point(world_pos, self.world_position())
    }

    /// Apply blocking to navigation grid (default implementation)
    fn apply_blocking(&self, nav_grid: &mut NavigationGrid) {
        if !self.blocks_pathfinding() {
            return;
        }

        self.collision_shape().block_navigation_grid(
            nav_grid,
            self.world_position(),
            self.blocking_priority(),
        );
    }
}

/// Type-erased obstacle for collections
pub type BoxedObstacle = Box<dyn Obstacle>;

#[cfg(test)]
mod tests {
    #[test]
    fn test_obstacle_trait_basics() {
        // Basic trait functionality is tested with concrete implementations
        // All obstacle system functionality is comprehensively tested in submodules
    }
}
