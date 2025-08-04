//! Centralized obstacle management for pathfinding

use crate::map::EnvironmentObject;
use crate::pathfinding::{NavigationGrid, obstacles::*};

/// Manages obstacles for efficient pathfinding integration
#[derive(Default)]
pub struct ObstacleManager {
    static_obstacles: Vec<Box<dyn Obstacle>>,
    dynamic_obstacles: Vec<Box<dyn Obstacle>>,
    needs_rebuild: bool,
}

impl ObstacleManager {
    pub fn new() -> Self {
        Self {
            static_obstacles: Vec::new(),
            dynamic_obstacles: Vec::new(),
            needs_rebuild: false,
        }
    }

    /// Add static obstacle from environment object
    pub fn add_environment_obstacle(&mut self, env_obj: &EnvironmentObject) {
        let obstacle = EnvironmentObstacle::from(env_obj);
        self.static_obstacles.push(Box::new(obstacle));
        self.needs_rebuild = true;
    }

    /// Add multiple environment obstacles efficiently
    pub fn add_environment_obstacles(&mut self, env_objects: &[EnvironmentObject]) {
        for obj in env_objects {
            let obstacle = EnvironmentObstacle::from(obj);
            self.static_obstacles.push(Box::new(obstacle));
        }
        self.needs_rebuild = true;
    }

    /// Add dynamic obstacle from entity
    pub fn add_entity_obstacle(
        &mut self,
        entity: Entity,
        position: Vec3,
        radius: f32,
        obstacle_type: EntityObstacleType,
    ) {
        let obstacle = EntityObstacle::new(entity, position, radius, obstacle_type);
        self.dynamic_obstacles.push(Box::new(obstacle));
    }

    /// Add dynamic obstacle from EntityObstacle
    pub fn add_dynamic_obstacle(&mut self, obstacle: EntityObstacle) {
        self.dynamic_obstacles.push(Box::new(obstacle));
    }

    /// Clear all dynamic obstacles (called each frame/update)
    pub fn clear_dynamic_obstacles(&mut self) {
        self.dynamic_obstacles.clear();
    }

    /// Clear all static obstacles
    pub fn clear_static_obstacles(&mut self) {
        self.static_obstacles.clear();
        self.needs_rebuild = true;
    }

    /// Apply all obstacles to navigation grid
    pub fn apply_to_navigation_grid(&self, nav_grid: &mut NavigationGrid) {
        // Apply static obstacles first
        for obstacle in &self.static_obstacles {
            obstacle.apply_blocking(nav_grid);
        }

        // Apply dynamic obstacles (may override static based on priority)
        for obstacle in &self.dynamic_obstacles {
            obstacle.apply_blocking(nav_grid);
        }
    }

    /// Get count of obstacles by type
    pub fn obstacle_counts(&self) -> (usize, usize) {
        (self.static_obstacles.len(), self.dynamic_obstacles.len())
    }

    /// Check if static obstacles need rebuilding
    pub fn needs_static_rebuild(&self) -> bool {
        self.needs_rebuild
    }

    /// Mark static obstacles as rebuilt
    pub fn mark_rebuilt(&mut self) {
        self.needs_rebuild = false;
    }

    /// Clear all obstacles
    pub fn clear_all(&mut self) {
        self.static_obstacles.clear();
        self.dynamic_obstacles.clear();
        self.needs_rebuild = true;
    }

    /// Get all static obstacles (for iteration/debugging)
    pub fn static_obstacles(&self) -> &[Box<dyn Obstacle>] {
        &self.static_obstacles
    }

    /// Get all dynamic obstacles (for iteration/debugging)
    pub fn dynamic_obstacles(&self) -> &[Box<dyn Obstacle>] {
        &self.dynamic_obstacles
    }

    /// Check if any obstacle at position blocks pathfinding
    pub fn is_position_blocked(&self, position: Vec3) -> bool {
        // Check static obstacles first
        for obstacle in &self.static_obstacles {
            if obstacle.blocks_pathfinding() && obstacle.contains_point(position) {
                return true;
            }
        }

        // Check dynamic obstacles
        for obstacle in &self.dynamic_obstacles {
            if obstacle.blocks_pathfinding() && obstacle.contains_point(position) {
                return true;
            }
        }

        false
    }

    /// Get the highest priority obstacle at a position
    pub fn get_highest_priority_at_position(&self, position: Vec3) -> Option<u8> {
        let mut highest_priority = None;

        // Check static obstacles
        for obstacle in &self.static_obstacles {
            if obstacle.blocks_pathfinding() && obstacle.contains_point(position) {
                let priority = obstacle.blocking_priority();
                highest_priority = Some(highest_priority.unwrap_or(0).max(priority));
            }
        }

        // Check dynamic obstacles
        for obstacle in &self.dynamic_obstacles {
            if obstacle.blocks_pathfinding() && obstacle.contains_point(position) {
                let priority = obstacle.blocking_priority();
                highest_priority = Some(highest_priority.unwrap_or(0).max(priority));
            }
        }

        highest_priority
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::TerrainData;
    use crate::pathfinding::PathfindingConfig;

    #[test]
    fn test_obstacle_manager_basic_operations() {
        let mut manager = ObstacleManager::new();

        // Add environment obstacle
        let env_obj = EnvironmentObject::simple("tree".to_string(), Vec3::new(5.0, 0.0, 5.0));
        manager.add_environment_obstacle(&env_obj);

        let (static_count, dynamic_count) = manager.obstacle_counts();
        assert_eq!(static_count, 1);
        assert_eq!(dynamic_count, 0);
        assert!(manager.needs_static_rebuild());

        // Add entity obstacle
        let entity = Entity::from_raw(456);
        manager.add_entity_obstacle(
            entity,
            Vec3::new(3.0, 0.0, 3.0),
            0.5,
            EntityObstacleType::Player,
        );

        let (static_count, dynamic_count) = manager.obstacle_counts();
        assert_eq!(static_count, 1);
        assert_eq!(dynamic_count, 1);

        // Clear dynamics
        manager.clear_dynamic_obstacles();
        let (static_count, dynamic_count) = manager.obstacle_counts();
        assert_eq!(static_count, 1);
        assert_eq!(dynamic_count, 0);
    }

    #[test]
    fn test_obstacle_manager_multiple_environment_obstacles() {
        let mut manager = ObstacleManager::new();

        let objects = vec![
            EnvironmentObject::new(
                "tree".to_string(),
                Vec3::new(1.0, 0.0, 1.0),
                Vec3::ZERO,
                Vec3::new(2.0, 3.0, 2.0),
            ),
            EnvironmentObject::new(
                "rock".to_string(),
                Vec3::new(-1.0, 0.0, -1.0),
                Vec3::ZERO,
                Vec3::new(1.0, 1.0, 1.0),
            ),
            EnvironmentObject::new(
                "grass".to_string(),
                Vec3::new(2.0, 0.0, -2.0),
                Vec3::ZERO,
                Vec3::ONE,
            ),
        ];

        manager.add_environment_obstacles(&objects);

        let (static_count, _) = manager.obstacle_counts();
        assert_eq!(static_count, 3);
        assert!(manager.needs_static_rebuild());
    }

    #[test]
    fn test_obstacle_manager_navigation_grid_integration() {
        let terrain = TerrainData::create_flat(8, 8, 1.0, 0.0).unwrap();
        let mut nav_grid =
            NavigationGrid::from_terrain(&terrain, PathfindingConfig::default()).unwrap();

        let mut manager = ObstacleManager::new();

        // Add tree obstacle
        let tree = EnvironmentObject::new(
            "tree".to_string(),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::ZERO,
            Vec3::new(2.0, 3.0, 2.0),
        );
        manager.add_environment_obstacle(&tree);

        // Apply to grid
        manager.apply_to_navigation_grid(&mut nav_grid);

        // Check that tree location is blocked
        let tree_node = nav_grid.world_to_grid(Vec3::new(0.0, 0.0, 0.0)).unwrap();
        assert!(!nav_grid.is_walkable(tree_node.x, tree_node.z));

        manager.mark_rebuilt();
        assert!(!manager.needs_static_rebuild());
    }

    #[test]
    fn test_obstacle_manager_position_blocking_check() {
        let mut manager = ObstacleManager::new();

        // Add a tree that blocks pathfinding
        let tree = EnvironmentObject::new(
            "tree".to_string(),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::ZERO,
            Vec3::new(2.0, 3.0, 2.0), // Radius will be 2.0 * 0.3 * 2.0 = 1.2
        );
        manager.add_environment_obstacle(&tree);

        // Add grass that doesn't block pathfinding
        let grass = EnvironmentObject::new(
            "grass".to_string(),
            Vec3::new(5.0, 0.0, 5.0),
            Vec3::ZERO,
            Vec3::ONE,
        );
        manager.add_environment_obstacle(&grass);

        // Position inside tree should be blocked
        assert!(manager.is_position_blocked(Vec3::new(0.3, 0.0, 0.0)));

        // Position outside tree should not be blocked (now needs to be further due to larger radius)
        assert!(!manager.is_position_blocked(Vec3::new(1.5, 0.0, 0.0)));

        // Position at grass should not be blocked (grass doesn't block pathfinding)
        assert!(!manager.is_position_blocked(Vec3::new(5.0, 0.0, 5.0)));
    }

    #[test]
    fn test_obstacle_manager_priority_detection() {
        let mut manager = ObstacleManager::new();

        // Add tree (priority 150)
        let tree = EnvironmentObject::new(
            "tree".to_string(),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::ZERO,
            Vec3::new(2.0, 3.0, 2.0),
        );
        manager.add_environment_obstacle(&tree);

        // Add dynamic player obstacle (priority 180) at distant position
        let entity = Entity::from_raw(123);
        manager.add_entity_obstacle(
            entity,
            Vec3::new(5.0, 0.0, 5.0), // Far from tree
            0.5,
            EntityObstacleType::Player,
        );

        // At tree position, should get tree priority only
        assert_eq!(
            manager.get_highest_priority_at_position(Vec3::new(0.0, 0.0, 0.0)),
            Some(150)
        );

        // At player position, should get player priority (higher)
        assert_eq!(
            manager.get_highest_priority_at_position(Vec3::new(5.0, 0.0, 5.0)),
            Some(180)
        );

        // At empty position, should get None
        assert_eq!(
            manager.get_highest_priority_at_position(Vec3::new(10.0, 0.0, 10.0)),
            None
        );
    }

    #[test]
    fn test_obstacle_manager_clear_operations() {
        let mut manager = ObstacleManager::new();

        // Add some obstacles
        let tree = EnvironmentObject::simple("tree".to_string(), Vec3::new(1.0, 0.0, 1.0));
        manager.add_environment_obstacle(&tree);

        let entity = Entity::from_raw(789);
        manager.add_entity_obstacle(
            entity,
            Vec3::new(2.0, 0.0, 2.0),
            0.5,
            EntityObstacleType::Enemy,
        );

        let (static_count, dynamic_count) = manager.obstacle_counts();
        assert_eq!(static_count, 1);
        assert_eq!(dynamic_count, 1);

        // Clear just static obstacles
        manager.clear_static_obstacles();
        let (static_count, dynamic_count) = manager.obstacle_counts();
        assert_eq!(static_count, 0);
        assert_eq!(dynamic_count, 1);
        assert!(manager.needs_static_rebuild());

        // Re-add static obstacle
        manager.add_environment_obstacle(&tree);

        // Clear all obstacles
        manager.clear_all();
        let (static_count, dynamic_count) = manager.obstacle_counts();
        assert_eq!(static_count, 0);
        assert_eq!(dynamic_count, 0);
        assert!(manager.needs_static_rebuild());
    }

    #[test]
    fn test_dynamic_obstacle_creation() {
        let mut manager = ObstacleManager::new();

        let entity = Entity::from_raw(999);
        let dynamic_obstacle = EntityObstacle::player(entity, Vec3::new(3.0, 0.0, 3.0), 0.7);

        manager.add_dynamic_obstacle(dynamic_obstacle);

        let (static_count, dynamic_count) = manager.obstacle_counts();
        assert_eq!(static_count, 0);
        assert_eq!(dynamic_count, 1);
    }
}
