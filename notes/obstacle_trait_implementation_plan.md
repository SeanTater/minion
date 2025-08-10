# Obstacle Trait Implementation Plan

## File Structure & Module Organization

```
src/pathfinding/
├── mod.rs (existing - update exports)
├── integration_example.rs (existing)
├── obstacles/
│   ├── mod.rs (new - trait definitions)
│   ├── collision_shapes.rs (new - geometric shapes)
│   ├── environment_obstacles.rs (new - static obstacles)
│   ├── entity_obstacles.rs (new - dynamic obstacles)
│   └── obstacle_manager.rs (new - centralized management)
└── grid_blocking.rs (new - optimized grid operations)
```

## Phase 1: Core Infrastructure (1-2 days)

### Step 1.1: Create Obstacle Trait Module

**File: `src/pathfinding/obstacles/mod.rs`**
```rust
//! Trait-based obstacle system for pathfinding collision detection

use bevy::prelude::*;
use crate::pathfinding::{NavigationGrid, GridNode};

pub mod collision_shapes;
pub mod environment_obstacles;
pub mod entity_obstacles;
pub mod obstacle_manager;

pub use collision_shapes::*;
pub use environment_obstacles::*;
pub use entity_obstacles::*;
pub use obstacle_manager::*;

/// Core trait for objects that can obstruct pathfinding
pub trait Obstacle: Send + Sync {
    /// Get the obstacle's collision shape for pathfinding
    fn collision_shape(&self) -> CollisionShape;

    /// Check if this obstacle blocks pathfinding at all
    fn blocks_pathfinding(&self) -> bool { true }

    /// Get obstacle priority for overlapping obstacles (higher = more important)
    fn blocking_priority(&self) -> u8 { 100 }

    /// Get the world position of this obstacle
    fn world_position(&self) -> Vec3;

    /// Test if a world position is inside this obstacle
    fn contains_point(&self, world_pos: Vec3) -> bool {
        self.collision_shape().contains_point(world_pos, self.world_position())
    }

    /// Apply blocking to navigation grid (default implementation)
    fn apply_blocking(&self, nav_grid: &mut NavigationGrid) {
        if !self.blocks_pathfinding() {
            return;
        }

        self.collision_shape().block_navigation_grid(
            nav_grid,
            self.world_position(),
            self.blocking_priority()
        );
    }
}

/// Type-erased obstacle for collections
pub type BoxedObstacle = Box<dyn Obstacle>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::TerrainData;

    #[test]
    fn test_obstacle_trait_basics() {
        // Tests will be added during implementation
    }
}
```

### Step 1.2: Create Collision Shapes

**File: `src/pathfinding/obstacles/collision_shapes.rs`**
```rust
//! Geometric collision shapes for obstacle detection

use bevy::prelude::*;
use crate::pathfinding::{NavigationGrid, GridNode};

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
                let distance_2d = Vec2::new(
                    world_pos.x - shape_center.x,
                    world_pos.z - shape_center.z
                ).length();
                distance_2d <= *radius
            }
            CollisionShape::Rectangle { half_extents } => {
                let rel_pos = world_pos - shape_center;
                rel_pos.x.abs() <= half_extents.x &&
                rel_pos.z.abs() <= half_extents.z
            }
            CollisionShape::Capsule { radius, .. } => {
                // Treat as circle for 2D pathfinding
                let distance_2d = Vec2::new(
                    world_pos.x - shape_center.x,
                    world_pos.z - shape_center.z
                ).length();
                distance_2d <= *radius
            }
            CollisionShape::Compound { shapes } => {
                shapes.iter().any(|(offset, shape)| {
                    shape.contains_point(world_pos, shape_center + *offset)
                })
            }
            CollisionShape::None => false,
        }
    }

    /// Apply this shape to the navigation grid
    pub fn block_navigation_grid(&self, nav_grid: &mut NavigationGrid, center: Vec3, priority: u8) {
        match self {
            CollisionShape::Circle { radius } => {
                crate::pathfinding::grid_blocking::block_circular_area_with_priority(
                    nav_grid, center, *radius, priority
                );
            }
            CollisionShape::Rectangle { half_extents } => {
                crate::pathfinding::grid_blocking::block_rectangular_area_with_priority(
                    nav_grid, center, *half_extents, priority
                );
            }
            CollisionShape::Capsule { radius, .. } => {
                crate::pathfinding::grid_blocking::block_circular_area_with_priority(
                    nav_grid, center, *radius, priority
                );
            }
            CollisionShape::Compound { shapes } => {
                for (offset, shape) in shapes {
                    shape.block_navigation_grid(nav_grid, center + *offset, priority);
                }
            }
            CollisionShape::None => {},
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
            half_extents: Vec3::new(2.0, 1.0, 2.0)
        };
        let center = Vec3::new(0.0, 0.0, 0.0);

        assert!(shape.contains_point(Vec3::new(1.5, 0.0, 1.5), center));
        assert!(!shape.contains_point(Vec3::new(2.5, 0.0, 0.0), center));
    }
}
```

### Step 1.3: Create Grid Blocking Optimizations

**File: `src/pathfinding/grid_blocking.rs`**
```rust
//! Optimized grid blocking operations with priority support

use bevy::prelude::*;
use crate::pathfinding::{NavigationGrid, GridNode};

/// Block circular area with priority-based override system
pub fn block_circular_area_with_priority(
    nav_grid: &mut NavigationGrid,
    center: Vec3,
    radius: f32,
    priority: u8
) {
    let Some(center_cell) = nav_grid.world_to_grid(center) else { return };
    let cell_radius = (radius / nav_grid.cell_size).ceil() as i32;

    // Pre-calculate constants for performance
    let half_width = (nav_grid.terrain_width as f32 * nav_grid.terrain_scale) / 2.0;
    let half_height = (nav_grid.terrain_height as f32 * nav_grid.terrain_scale) / 2.0;
    let radius_squared = radius * radius;

    for dz in -cell_radius..=cell_radius {
        for dx in -cell_radius..=cell_radius {
            let x = center_cell.x as i32 + dx;
            let z = center_cell.z as i32 + dz;

            if x >= 0 && z >= 0 && x < nav_grid.width as i32 && z < nav_grid.height as i32 {
                // Fast squared distance check
                let world_x = (x as f32 * nav_grid.terrain_scale) - half_width;
                let world_z = (z as f32 * nav_grid.terrain_scale) - half_height;
                let dx_world = world_x - center.x;
                let dz_world = world_z - center.z;

                if dx_world * dx_world + dz_world * dz_world <= radius_squared {
                    let cell = GridNode::new(x as u32, z as u32);
                    nav_grid.set_cell_walkable_with_priority(cell, false, priority);
                }
            }
        }
    }
}

/// Block rectangular area with priority support
pub fn block_rectangular_area_with_priority(
    nav_grid: &mut NavigationGrid,
    center: Vec3,
    half_extents: Vec3,
    priority: u8
) {
    let min_world = center - half_extents;
    let max_world = center + half_extents;

    let Some(min_cell) = nav_grid.world_to_grid(min_world) else { return };
    let Some(max_cell) = nav_grid.world_to_grid(max_world) else { return };

    for z in min_cell.z..=max_cell.z {
        for x in min_cell.x..=max_cell.x {
            nav_grid.set_cell_walkable_with_priority(GridNode::new(x, z), false, priority);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::TerrainData;
    use crate::pathfinding::PathfindingConfig;

    #[test]
    fn test_circular_blocking_with_priority() {
        let terrain = TerrainData::create_flat(8, 8, 1.0, 0.0).unwrap();
        let mut nav_grid = NavigationGrid::from_terrain(&terrain, PathfindingConfig::default()).unwrap();

        // Block with lower priority first
        block_circular_area_with_priority(&mut nav_grid, Vec3::ZERO, 1.5, 100);
        let center_node = nav_grid.world_to_grid(Vec3::ZERO).unwrap();
        assert!(!nav_grid.is_walkable(center_node.x, center_node.z));

        // Try to override with higher priority - should work
        block_circular_area_with_priority(&mut nav_grid, Vec3::ZERO, 0.5, 200);
        // (Would need to implement walkable override for this test)
    }
}
```

## Phase 2: NavigationGrid Extensions (1 day)

### Step 2.1: Extend NavigationGrid with Priority System

**Update: `src/pathfinding/mod.rs`**
```rust
// Add to NavigationGrid struct
pub struct NavigationGrid {
    // ... existing fields ...
    /// Priority values for obstacle blocking (higher = more important)
    pub obstacle_priorities: Vec<u8>,
}

impl NavigationGrid {
    /// Create new grid with priority tracking
    pub fn from_terrain_and_objects(
        terrain: &TerrainData,
        objects: &[EnvironmentObject],
        config: PathfindingConfig
    ) -> MinionResult<Self> {
        let total_cells = (terrain.width * terrain.height) as usize;
        let mut walkable = Vec::with_capacity(total_cells);
        let mut heights = Vec::with_capacity(total_cells);
        let mut obstacle_priorities = vec![0u8; total_cells]; // Initialize priorities

        // ... existing terrain processing ...

        let mut nav_grid = NavigationGrid {
            walkable,
            heights,
            width: terrain.width,
            height: terrain.height,
            cell_size: terrain.scale,
            terrain_width: terrain.width,
            terrain_height: terrain.height,
            terrain_scale: terrain.scale,
            config,
            obstacle_priorities, // Add priority tracking
        };

        // NEW: Use trait-based obstacle system
        use crate::pathfinding::obstacles::*;
        let mut obstacle_manager = ObstacleManager::new();

        for obj in objects {
            obstacle_manager.add_environment_obstacle(obj);
        }

        obstacle_manager.apply_to_navigation_grid(&mut nav_grid);

        Ok(nav_grid)
    }

    /// Set cell walkability with priority system
    pub fn set_cell_walkable_with_priority(&mut self, node: GridNode, walkable: bool, priority: u8) {
        if node.x >= self.width || node.z >= self.height {
            return;
        }
        let index = (node.z * self.width + node.x) as usize;

        // Priority-based override system
        if !walkable {
            if let Some(current_priority) = self.obstacle_priorities.get(index) {
                if priority < *current_priority {
                    return; // Don't override higher priority obstacle
                }
            }
            self.obstacle_priorities[index] = priority;
        } else {
            // When making walkable, only allow if priority is high enough
            if let Some(current_priority) = self.obstacle_priorities.get(index) {
                if priority < *current_priority {
                    return;
                }
            }
            self.obstacle_priorities[index] = 0; // Reset priority
        }

        if let Some(cell) = self.walkable.get_mut(index) {
            *cell = walkable;
        }
    }

    /// Get obstacle priority at cell
    pub fn get_obstacle_priority(&self, node: GridNode) -> u8 {
        if node.x >= self.width || node.z >= self.height {
            return 0;
        }
        let index = (node.z * self.width + node.x) as usize;
        self.obstacle_priorities.get(index).copied().unwrap_or(0)
    }
}
```

## Phase 3: Obstacle Implementations (2 days)

### Step 3.1: Environment Obstacles

**File: `src/pathfinding/obstacles/environment_obstacles.rs`**
```rust
//! Static environment obstacles from map data

use bevy::prelude::*;
use crate::map::EnvironmentObject;
use crate::pathfinding::obstacles::{Obstacle, CollisionShape};

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
    Tree { trunk_radius_factor: f32 },
    Rock { collision_factor: f32 },
    Boulder { collision_factor: f32 },
    Grass,
    Structure { collision_shape: CollisionShape },
    Custom { name: String, collision_shape: CollisionShape },
}

impl From<&EnvironmentObject> for EnvironmentObstacle {
    fn from(obj: &EnvironmentObject) -> Self {
        let object_type = match obj.object_type.as_str() {
            "tree" => EnvironmentObjectType::Tree { trunk_radius_factor: 0.3 },
            "rock" => EnvironmentObjectType::Rock { collision_factor: 0.5 },
            "boulder" => EnvironmentObjectType::Boulder { collision_factor: 0.5 },
            "grass" => EnvironmentObjectType::Grass,
            name => {
                // Default fallback for unknown types
                let collision_shape = if obj.scale.x > 0.0 && obj.scale.z > 0.0 {
                    CollisionShape::Rectangle { half_extents: obj.scale * 0.5 }
                } else {
                    CollisionShape::None
                };
                EnvironmentObjectType::Custom {
                    name: name.to_string(),
                    collision_shape
                }
            },
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
        match &self.object_type {
            EnvironmentObjectType::Tree { trunk_radius_factor } => {
                CollisionShape::Circle { radius: self.scale.x * trunk_radius_factor }
            }
            EnvironmentObjectType::Rock { collision_factor } |
            EnvironmentObjectType::Boulder { collision_factor } => {
                CollisionShape::Circle { radius: self.scale.x * collision_factor }
            }
            EnvironmentObjectType::Grass => CollisionShape::None,
            EnvironmentObjectType::Structure { collision_shape } => collision_shape.clone(),
            EnvironmentObjectType::Custom { collision_shape, .. } => collision_shape.clone(),
        }
    }

    fn blocks_pathfinding(&self) -> bool {
        !matches!(self.object_type, EnvironmentObjectType::Grass)
    }

    fn world_position(&self) -> Vec3 {
        self.position
    }

    fn blocking_priority(&self) -> u8 {
        match &self.object_type {
            EnvironmentObjectType::Tree { .. } => 150,
            EnvironmentObjectType::Boulder { .. } => 200,
            EnvironmentObjectType::Rock { .. } => 120,
            EnvironmentObjectType::Structure { .. } => 255,
            EnvironmentObjectType::Custom { .. } => 100,
            EnvironmentObjectType::Grass => 0,
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
        assert!(matches!(obstacle.object_type, EnvironmentObjectType::Tree { .. }));
        assert!(obstacle.blocks_pathfinding());
        assert_eq!(obstacle.blocking_priority(), 150);

        // Check collision shape
        match obstacle.collision_shape() {
            CollisionShape::Circle { radius } => {
                assert_eq!(radius, 2.0 * 0.3); // scale.x * trunk_radius_factor
            }
            _ => panic!("Expected circle collision shape for tree"),
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
        assert!(matches!(obstacle.object_type, EnvironmentObjectType::Custom { .. }));
        assert!(obstacle.blocks_pathfinding());

        match obstacle.collision_shape() {
            CollisionShape::Rectangle { half_extents } => {
                assert_eq!(half_extents, Vec3::new(2.0, 3.0, 2.0)); // scale * 0.5
            }
            _ => panic!("Expected rectangle collision shape for custom object"),
        }
    }
}
```

### Step 3.2: Entity Obstacles

**File: `src/pathfinding/obstacles/entity_obstacles.rs`**
```rust
//! Dynamic entity obstacles for moving objects

use bevy::prelude::*;
use crate::pathfinding::obstacles::{Obstacle, CollisionShape};

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
    pub fn new(entity_id: Entity, position: Vec3, collision_radius: f32, obstacle_type: EntityObstacleType) -> Self {
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
}

impl Obstacle for EntityObstacle {
    fn collision_shape(&self) -> CollisionShape {
        CollisionShape::Circle { radius: self.collision_radius }
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
    fn test_obstacle_source_component() {
        let source = ObstacleSource::enemy(0.8);

        assert_eq!(source.collision_radius, 0.8);
        assert!(matches!(source.obstacle_type, EntityObstacleType::Enemy));
        assert!(source.blocks_pathfinding);
    }
}
```

### Step 3.3: Obstacle Manager

**File: `src/pathfinding/obstacles/obstacle_manager.rs`**
```rust
//! Centralized obstacle management for pathfinding

use bevy::prelude::*;
use crate::map::EnvironmentObject;
use crate::pathfinding::{NavigationGrid, obstacles::*};

/// Manages obstacles for efficient pathfinding integration
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
    pub fn add_entity_obstacle(&mut self, entity: Entity, position: Vec3, radius: f32, obstacle_type: EntityObstacleType) {
        let obstacle = EntityObstacle::new(entity, position, radius, obstacle_type);
        self.dynamic_obstacles.push(Box::new(obstacle));
    }

    /// Clear all dynamic obstacles (called each frame/update)
    pub fn clear_dynamic_obstacles(&mut self) {
        self.dynamic_obstacles.clear();
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
}

impl Default for ObstacleManager {
    fn default() -> Self {
        Self::new()
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
        manager.add_entity_obstacle(entity, Vec3::new(3.0, 0.0, 3.0), 0.5, EntityObstacleType::Player);

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
    fn test_obstacle_manager_navigation_grid_integration() {
        let terrain = TerrainData::create_flat(8, 8, 1.0, 0.0).unwrap();
        let mut nav_grid = NavigationGrid::from_terrain(&terrain, PathfindingConfig::default()).unwrap();

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
}
```

## Phase 4: Integration & Testing (1 day)

### Step 4.1: Update Module Exports

**Update: `src/pathfinding/mod.rs`**
```rust
// Add to existing exports
pub mod obstacles;
pub mod grid_blocking;

pub use obstacles::*;
```

### Step 4.2: Create Migration Test

**File: `src/pathfinding/migration_test.rs`**
```rust
//! Tests to ensure trait-based system maintains backward compatibility

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::{TerrainData, EnvironmentObject};
    use crate::pathfinding::{NavigationGrid, PathfindingConfig};

    /// Test that new trait system produces same results as old system
    #[test]
    fn test_backward_compatibility() {
        let terrain = TerrainData::create_flat(16, 16, 1.0, 0.0).unwrap();
        let objects = vec![
            EnvironmentObject::new("tree".to_string(), Vec3::new(2.0, 0.0, 2.0), Vec3::ZERO, Vec3::new(2.0, 3.0, 2.0)),
            EnvironmentObject::new("rock".to_string(), Vec3::new(-2.0, 0.0, -2.0), Vec3::ZERO, Vec3::new(1.0, 1.0, 1.0)),
            EnvironmentObject::new("grass".to_string(), Vec3::new(4.0, 0.0, -4.0), Vec3::ZERO, Vec3::ONE),
            EnvironmentObject::new("unknown_type".to_string(), Vec3::new(-4.0, 0.0, 4.0), Vec3::ZERO, Vec3::new(1.5, 2.0, 1.5)),
        ];

        // Create navigation grid with new trait system
        let nav_grid = NavigationGrid::from_terrain_and_objects(&terrain, &objects, PathfindingConfig::default()).unwrap();

        // Verify expected blocking behavior

        // Tree should be blocked (circular area)
        let tree_node = nav_grid.world_to_grid(Vec3::new(2.0, 0.0, 2.0)).unwrap();
        assert!(!nav_grid.is_walkable(tree_node.x, tree_node.z));

        // Rock should be blocked (circular area)
        let rock_node = nav_grid.world_to_grid(Vec3::new(-2.0, 0.0, -2.0)).unwrap();
        assert!(!nav_grid.is_walkable(rock_node.x, rock_node.z));

        // Grass should NOT be blocked
        let grass_node = nav_grid.world_to_grid(Vec3::new(4.0, 0.0, -4.0)).unwrap();
        assert!(nav_grid.is_walkable(grass_node.x, grass_node.z));

        // Unknown type should be blocked (rectangular area)
        let unknown_node = nav_grid.world_to_grid(Vec3::new(-4.0, 0.0, 4.0)).unwrap();
        assert!(!nav_grid.is_walkable(unknown_node.x, unknown_node.z));
    }

    #[test]
    fn test_performance_regression() {
        use std::time::Instant;

        let terrain = TerrainData::create_flat(64, 64, 1.0, 0.0).unwrap();

        // Create many objects to test performance
        let mut objects = Vec::new();
        for i in 0..100 {
            let x = (i % 10) as f32 * 2.0 - 10.0;
            let z = ((i / 10) % 10) as f32 * 2.0 - 10.0;
            objects.push(EnvironmentObject::new(
                "tree".to_string(),
                Vec3::new(x, 0.0, z),
                Vec3::ZERO,
                Vec3::new(1.5, 2.0, 1.5),
            ));
        }

        let start = Instant::now();
        let _nav_grid = NavigationGrid::from_terrain_and_objects(&terrain, &objects, PathfindingConfig::default()).unwrap();
        let duration = start.elapsed();

        // Should complete within reasonable time (adjust threshold as needed)
        assert!(duration.as_millis() < 100, "Grid generation took {}ms, expected <100ms", duration.as_millis());
    }
}
```

## Integration Checklist

### Before Implementation
- [ ] Review existing pathfinding system thoroughly
- [ ] Identify all current obstacle types and their behaviors
- [ ] Plan test cases for backward compatibility
- [ ] Set up performance benchmarks

### During Implementation
- [ ] Implement trait infrastructure first
- [ ] Add comprehensive unit tests for each component
- [ ] Test collision shape accuracy against manual calculations
- [ ] Verify priority system works correctly
- [ ] Benchmark performance against current system

### After Implementation
- [ ] Run full test suite including existing pathfinding tests
- [ ] Performance comparison with before/after metrics
- [ ] Integration test with real game scenarios
- [ ] Update documentation and examples
- [ ] Code review focusing on trait object overhead

## Performance Targets

- **Grid Generation**: 64x64 terrain with 100 obstacles < 50ms
- **Memory Overhead**: <10% increase from trait objects
- **Collision Testing**: Individual obstacle test < 1μs
- **Priority Resolution**: Handle 50+ overlapping obstacles efficiently

## Future Extensions

Once core system is stable:

1. **Spatial Optimization**: Add spatial hashing for large environments
2. **Conditional Blocking**: Obstacles that block some entities but not others
3. **Temporal Obstacles**: Time-based obstacle activation
4. **Complex Shapes**: Multi-part compound obstacles
5. **Real-time Updates**: Incremental navigation grid updates

This implementation plan provides a clear path from current string-based type matching to a robust, extensible trait-based system while maintaining performance and backward compatibility.
