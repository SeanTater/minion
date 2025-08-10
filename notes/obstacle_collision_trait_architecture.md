# Obstacle Collision Trait Architecture Design

## Executive Summary

This document outlines a comprehensive trait-based architecture for obstacle collision detection in our pathfinding system. The design replaces type-specific blocking functions with a polymorphic, extensible system that maintains performance while enabling easy addition of new obstacle types.

## Current System Analysis

### Existing Architecture Problems
```rust
// Current problematic approach in NavigationGrid::project_object_to_grid()
match obj.object_type.as_str() {
    "tree" => Self::block_circular_area(nav_grid, obj.position, obj.scale.x * 0.3),
    "rock" | "boulder" => Self::block_circular_area(nav_grid, obj.position, obj.scale.x * 0.5),
    "grass" => return, // No blocking
    _ => Self::block_rectangular_area(nav_grid, obj.position, obj.scale * 0.5),
}
```

**Issues:**
- String-based type matching is fragile and slow
- Adding new types requires modifying pathfinding core
- Coupling between obstacle types and pathfinding logic
- No compile-time type safety for obstacle behaviors
- Duplication of collision logic

### Performance Requirements
- Grid generation for 32x32 terrain with 50+ objects: ~1ms target
- Called frequently during navigation grid updates
- Must support efficient spatial queries
- Memory allocation should be minimal during collision testing

## Architecture Design

### 1. Core Trait Definition

```rust
/// Trait for objects that can obstruct pathfinding
pub trait Obstacle {
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

    /// Apply blocking to navigation grid (optimized for batch operations)
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
```

### 2. Collision Shape System

```rust
/// Geometric shapes for collision detection
#[derive(Debug, Clone)]
pub enum CollisionShape {
    Circle { radius: f32 },
    Rectangle { half_extents: Vec3 },
    Capsule { radius: f32, height: f32 },
    Compound { shapes: Vec<(Vec3, CollisionShape)> }, // offset + shape pairs
    None, // For non-blocking decorative objects
}

impl CollisionShape {
    /// Check if a world position is inside this shape
    pub fn contains_point(&self, world_pos: Vec3, shape_center: Vec3) -> bool {
        match self {
            CollisionShape::Circle { radius } => {
                let distance_2d = Vec2::new(world_pos.x - shape_center.x, world_pos.z - shape_center.z).length();
                distance_2d <= *radius
            }
            CollisionShape::Rectangle { half_extents } => {
                let rel_pos = world_pos - shape_center;
                rel_pos.x.abs() <= half_extents.x &&
                rel_pos.z.abs() <= half_extents.z
            }
            CollisionShape::Capsule { radius, height: _ } => {
                // For pathfinding, treat capsule as circle (2D pathfinding)
                let distance_2d = Vec2::new(world_pos.x - shape_center.x, world_pos.z - shape_center.z).length();
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

    /// Apply this shape to the navigation grid (optimized implementation)
    pub fn block_navigation_grid(&self, nav_grid: &mut NavigationGrid, center: Vec3, priority: u8) {
        match self {
            CollisionShape::Circle { radius } => {
                block_circular_area_optimized(nav_grid, center, *radius, priority);
            }
            CollisionShape::Rectangle { half_extents } => {
                block_rectangular_area_optimized(nav_grid, center, *half_extents, priority);
            }
            CollisionShape::Capsule { radius, height: _ } => {
                // Treat as circle for 2D pathfinding
                block_circular_area_optimized(nav_grid, center, *radius, priority);
            }
            CollisionShape::Compound { shapes } => {
                for (offset, shape) in shapes {
                    shape.block_navigation_grid(nav_grid, center + *offset, priority);
                }
            }
            CollisionShape::None => {}, // No blocking
        }
    }

    /// Get approximate bounds for this shape (used for spatial optimization)
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
```

### 3. Obstacle Type Implementations

```rust
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
    Grass, // Decorative only
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
            name => EnvironmentObjectType::Custom {
                name: name.to_string(),
                collision_shape: CollisionShape::Rectangle {
                    half_extents: obj.scale * 0.5
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
            EnvironmentObjectType::Tree { .. } => 150,     // Trees are important obstacles
            EnvironmentObjectType::Boulder { .. } => 200,  // Boulders are very solid
            EnvironmentObjectType::Rock { .. } => 120,     // Rocks are medium priority
            EnvironmentObjectType::Structure { .. } => 255, // Structures are absolute
            EnvironmentObjectType::Custom { .. } => 100,   // Default priority
            EnvironmentObjectType::Grass => 0,             // Grass doesn't block
        }
    }
}

/// Dynamic entity obstacle (for moving objects like players, enemies)
#[derive(Debug)]
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
```

### 4. High-Performance Grid Application

```rust
/// Optimized blocking functions with priority support
fn block_circular_area_optimized(nav_grid: &mut NavigationGrid, center: Vec3, radius: f32, priority: u8) {
    let Some(center_cell) = nav_grid.world_to_grid(center) else { return };
    let cell_radius = (radius / nav_grid.cell_size).ceil() as i32;

    // Pre-calculate world coordinate conversion constants
    let half_width = (nav_grid.terrain_width as f32 * nav_grid.terrain_scale) / 2.0;
    let half_height = (nav_grid.terrain_height as f32 * nav_grid.terrain_scale) / 2.0;
    let radius_squared = radius * radius;

    for dz in -cell_radius..=cell_radius {
        for dx in -cell_radius..=cell_radius {
            let x = center_cell.x as i32 + dx;
            let z = center_cell.z as i32 + dz;

            if x >= 0 && z >= 0 && x < nav_grid.width as i32 && z < nav_grid.height as i32 {
                // Fast distance check using squared distance
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

fn block_rectangular_area_optimized(nav_grid: &mut NavigationGrid, center: Vec3, half_extents: Vec3, priority: u8) {
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

// Extension to NavigationGrid
impl NavigationGrid {
    /// Set cell walkability with priority - higher priority obstacles override lower priority
    fn set_cell_walkable_with_priority(&mut self, node: GridNode, walkable: bool, priority: u8) {
        if node.x >= self.width || node.z >= self.height {
            return;
        }
        let index = (node.z * self.width + node.x) as usize;

        // Only override if this obstacle has higher or equal priority
        if !walkable {
            // For obstacle placement, higher priority wins
            if let Some(current_priority) = self.obstacle_priorities.get(index) {
                if priority < *current_priority {
                    return; // Don't override higher priority obstacle
                }
            }
            self.obstacle_priorities[index] = priority;
        }

        if let Some(cell) = self.walkable.get_mut(index) {
            *cell = walkable;
        }
    }
}
```

### 5. Trait Object Integration

```rust
/// Obstacle manager for efficient batch processing
pub struct ObstacleManager {
    static_obstacles: Vec<Box<dyn Obstacle + Send + Sync>>,
    dynamic_obstacles: Vec<Box<dyn Obstacle + Send + Sync>>,
    spatial_cache: Option<SpatialObstacleCache>,
}

impl ObstacleManager {
    pub fn new() -> Self {
        Self {
            static_obstacles: Vec::new(),
            dynamic_obstacles: Vec::new(),
            spatial_cache: None,
        }
    }

    /// Add static obstacle from environment object
    pub fn add_environment_obstacle(&mut self, env_obj: &EnvironmentObject) {
        let obstacle = EnvironmentObstacle::from(env_obj);
        self.static_obstacles.push(Box::new(obstacle));
        self.spatial_cache = None; // Invalidate cache
    }

    /// Add dynamic obstacle from entity
    pub fn add_entity_obstacle(&mut self, entity: Entity, position: Vec3, radius: f32, obstacle_type: EntityObstacleType) {
        let obstacle = EntityObstacle {
            entity_id: entity,
            position,
            collision_radius: radius,
            obstacle_type,
        };
        self.dynamic_obstacles.push(Box::new(obstacle));
    }

    /// Clear all dynamic obstacles (called each frame)
    pub fn clear_dynamic_obstacles(&mut self) {
        self.dynamic_obstacles.clear();
    }

    /// Apply all obstacles to navigation grid with priority system
    pub fn apply_to_navigation_grid(&self, nav_grid: &mut NavigationGrid) {
        // Apply static obstacles first (they have consistent priority)
        for obstacle in &self.static_obstacles {
            obstacle.apply_blocking(nav_grid);
        }

        // Apply dynamic obstacles (may override static based on priority)
        for obstacle in &self.dynamic_obstacles {
            obstacle.apply_blocking(nav_grid);
        }
    }

    /// Build spatial cache for fast queries (for future optimization)
    pub fn build_spatial_cache(&mut self, bounds: (Vec3, Vec3)) {
        // Implementation would use spatial hashing or similar
        // for O(1) obstacle queries in specific regions
    }
}
```

## Migration Strategy

### Phase 1: Infrastructure Setup
1. **Add trait definitions** - Create `Obstacle` trait and `CollisionShape` enum
2. **Extend NavigationGrid** - Add priority-based blocking system
3. **Create obstacle types** - Implement `EnvironmentObstacle` and `EntityObstacle`
4. **Unit tests** - Test individual obstacle behaviors

### Phase 2: Integration with Existing System
1. **Replace type matching** - Update `NavigationGrid::from_terrain_and_objects()`
2. **Add ObstacleManager** - Centralize obstacle management
3. **Backward compatibility** - Keep existing API working during transition
4. **Performance testing** - Ensure no regression in grid generation speed

### Phase 3: Dynamic Obstacle Support
1. **Entity obstacle integration** - Add moving obstacles to pathfinding
2. **Real-time updates** - Support incremental navigation grid updates
3. **Spatial optimization** - Add spatial cache for large environments

### Phase 4: Advanced Features
1. **Compound obstacles** - Support complex multi-shape obstacles
2. **Conditional blocking** - Obstacles that block some entity types but not others
3. **Temporal obstacles** - Time-based obstacle activation/deactivation

## Performance Optimization Strategies

### 1. Spatial Partitioning
```rust
/// Spatial hash grid for fast obstacle queries
pub struct SpatialObstacleCache {
    grid_size: f32,
    cells: HashMap<(i32, i32), Vec<usize>>, // cell -> obstacle indices
    obstacles: Vec<Box<dyn Obstacle + Send + Sync>>,
}
```

### 2. Batch Processing
- Process all obstacles of the same type together
- Use SIMD operations where possible for distance calculations
- Cache frequently accessed values (world coordinates, radii)

### 3. Early Termination
- Skip obstacles that are clearly outside navigation bounds
- Use bounding box tests before detailed collision checks
- Priority-based early exit for overlapping obstacles

### 4. Memory Layout Optimization
- Store obstacle data in cache-friendly structures
- Use object pools for temporary obstacle instances
- Minimize heap allocations during pathfinding

## Future Extensibility

### Adding New Obstacle Types
```rust
/// Example: Adding a new obstacle type
pub struct BridgeObstacle {
    pub start: Vec3,
    pub end: Vec3,
    pub width: f32,
    pub blocks_ground_units: bool,
    pub blocks_flying_units: bool,
}

impl Obstacle for BridgeObstacle {
    fn collision_shape(&self) -> CollisionShape {
        // Bridge is walkable surface, not blocking
        if self.blocks_ground_units {
            CollisionShape::Rectangle {
                half_extents: Vec3::new(self.width * 0.5, 0.0,
                    self.start.distance(self.end) * 0.5)
            }
        } else {
            CollisionShape::None
        }
    }

    fn world_position(&self) -> Vec3 {
        (self.start + self.end) * 0.5
    }
}
```

### Entity System Integration
```rust
/// Component for making entities act as obstacles
#[derive(Component)]
pub struct ObstacleSource {
    pub collision_radius: f32,
    pub obstacle_type: EntityObstacleType,
    pub blocks_pathfinding: bool,
}

/// System to update dynamic obstacles from entity positions
pub fn update_entity_obstacles(
    obstacle_entities: Query<(Entity, &Transform, &ObstacleSource)>,
    mut obstacle_manager: ResMut<ObstacleManager>,
) {
    obstacle_manager.clear_dynamic_obstacles();

    for (entity, transform, obstacle_source) in obstacle_entities.iter() {
        if obstacle_source.blocks_pathfinding {
            obstacle_manager.add_entity_obstacle(
                entity,
                transform.translation,
                obstacle_source.collision_radius,
                obstacle_source.obstacle_type.clone(),
            );
        }
    }
}
```

## Testing Strategy

### Unit Tests
- Test each obstacle type's collision detection
- Verify priority system works correctly
- Test edge cases (overlapping obstacles, boundary conditions)

### Performance Tests
- Benchmark grid generation with various obstacle counts
- Compare performance against current string-matching system
- Profile memory allocation patterns

### Integration Tests
- Test with actual game scenarios
- Verify pathfinding still works correctly
- Test dynamic obstacle updates

## Conclusion

This trait-based architecture provides:

1. **Type Safety** - Compile-time guarantees about obstacle behavior
2. **Performance** - Optimized collision testing with priority system
3. **Extensibility** - Easy addition of new obstacle types
4. **Maintainability** - Separation of concerns between obstacle types and pathfinding
5. **Future-Proof** - Support for complex scenarios like dynamic obstacles and conditional blocking

The design maintains backward compatibility while providing a clear migration path to a more robust and extensible system.
