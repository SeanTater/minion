# Pathfinding System Architecture Design

## Executive Summary

This document outlines a comprehensive pathfinding system for the Minion ARPG that integrates seamlessly with the existing physics-based movement architecture. The design prioritizes code reuse between player and enemies, testability, and performance while leveraging the existing A* pathfinding library and terrain system.

## Current Codebase Analysis

### Existing Assets:
- **A* Library**: `pathfinding = "4.10"` already in Cargo.toml
- **Terrain System**: Grid-based heightmap with biome support
- **Physics Movement**: `KinematicCharacterController` with force-based movement
- **Movement Logic**: Pure, testable movement calculations in `src/game_logic/movement.rs`
- **Path Infrastructure**: Basic pathfinding in `src/terrain/path_generator.rs` (currently for terrain generation)

### Key Constraints:
- Physics-based movement using `RigidBody::Dynamic` with high damping (3.0 linear, 8.0 angular)
- Capsule colliders with locked rotation axes
- Existing enemy flocking system using separation forces
- Shared movement logic between player and enemies

## Overall System Architecture

### Core Components

```rust
// Core pathfinding components
#[derive(Component)]
pub struct PathfindingAgent {
    pub destination: Option<Vec3>,
    pub current_path: Option<NavPath>,
    pub path_progress: usize,
    pub replanning_timer: f32,
    pub agent_config: PathfindingConfig,
}

#[derive(Component)]
pub struct NavigationTarget {
    pub position: Vec3,
    pub tolerance: f32,
    pub priority: PathPriority,
}

#[derive(Component)]
pub struct PathfindingObstacle {
    pub radius: f32,
    pub is_dynamic: bool,
}

// Shared pathfinding logic (not a component)
pub struct NavPath {
    pub waypoints: Vec<Vec3>,
    pub created_at: f32,
    pub path_type: PathType,
}

#[derive(Resource)]
pub struct NavigationMesh {
    pub grid: NavGrid,
    pub static_obstacles: Vec<Obstacle>,
    pub last_update: f32,
}
```

### System Organization

```rust
// Core pathfinding systems
pub struct PathfindingPlugin;

impl Plugin for PathfindingPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(NavigationMesh::new())
            .add_systems(Update, (
                // High priority: path planning
                update_navigation_targets,
                plan_paths.after(update_navigation_targets),
                
                // Medium priority: path following
                follow_paths.after(plan_paths),
                
                // Low priority: maintenance
                replan_invalidated_paths,
                update_dynamic_obstacles,
            ).run_if(in_state(GameState::Playing)));
    }
}
```

## World Representation Strategy

### Grid-Based Navigation (Recommended Approach)

**Decision Rationale**: Given the existing terrain system uses a grid-based heightmap and the A* library is already available, a grid-based approach offers the best integration with minimal complexity.

```rust
#[derive(Resource)]
pub struct NavGrid {
    pub width: u32,
    pub height: u32,
    pub cell_size: f32,  // World units per cell (matches terrain scale)
    pub cells: Vec<NavCell>,
    pub bounds: (Vec3, Vec3),  // World space bounds
}

#[derive(Clone, Copy)]
pub struct NavCell {
    pub elevation: f32,
    pub traversable: bool,
    pub movement_cost: u8,  // 1-255, where 255 = nearly impassable
    pub biome_modifier: f32,
}

impl NavGrid {
    pub fn from_terrain(terrain: &TerrainData, obstacles: &[Obstacle]) -> Self {
        // Convert terrain heightmap to navigation grid
        // Apply obstacle masks
        // Calculate movement costs based on slope, biome, etc.
    }
    
    pub fn world_to_grid(&self, world_pos: Vec3) -> Option<(u32, u32)> {
        // Convert world coordinates to grid coordinates
    }
    
    pub fn grid_to_world(&self, grid_x: u32, grid_z: u32) -> Vec3 {
        // Convert grid coordinates to world center point
    }
}
```

### Alternative Approaches Considered:

1. **Navmesh**: More memory efficient for large worlds, but complex integration with existing terrain system
2. **Hierarchical A***: Better for large worlds, but adds complexity
3. **Flow Fields**: Excellent for large groups, but less suitable for individual pathfinding

## Integration with Physics Movement System

### Seamless Integration Strategy

The pathfinding system integrates with the existing movement system by providing waypoints to the existing `calculate_movement` function:

```rust
// Enhanced movement system
pub fn calculate_pathfinding_movement(
    current_position: Vec3,
    path: Option<&NavPath>,
    path_progress: usize,
    config: MovementConfig,
) -> (MovementCalculation, usize) {
    let target = path
        .and_then(|p| p.waypoints.get(path_progress))
        .copied();
    
    let movement = calculate_movement(current_position, target, config);
    
    // Advance to next waypoint if close enough to current one
    let new_progress = if movement.distance_to_target <= config.stopping_distance {
        path_progress + 1
    } else {
        path_progress
    };
    
    (movement, new_progress)
}
```

### Physics Force Integration

```rust
fn follow_paths(
    mut agents: Query<(
        &Transform, 
        &mut PathfindingAgent, 
        &mut KinematicCharacterController
    )>,
    time: Res<Time>,
    game_config: Res<GameConfig>,
) {
    for (transform, mut agent, mut controller) in agents.iter_mut() {
        if let Some(ref path) = agent.current_path {
            let config = MovementConfig {
                speed: agent.agent_config.max_speed,
                stopping_distance: agent.agent_config.waypoint_tolerance,
                slowdown_distance: agent.agent_config.slowdown_distance,
                delta_time: time.delta_secs(),
            };
            
            let (movement, new_progress) = calculate_pathfinding_movement(
                transform.translation,
                Some(path),
                agent.path_progress,
                config,
            );
            
            if movement.should_move {
                // Apply movement through physics system
                let movement_with_gravity = Vec3::new(
                    movement.movement_vector.x,
                    -3.0 * time.delta_secs(),  // Gravity
                    movement.movement_vector.z,
                );
                controller.translation = Some(movement_with_gravity);
                
                // Handle rotation
                if let Some(rotation_target) = movement.rotation_target {
                    // Apply rotation through transform (consistent with existing system)
                }
            }
            
            agent.path_progress = new_progress;
            
            // Clear path if completed
            if agent.path_progress >= path.waypoints.len() {
                agent.current_path = None;
                agent.destination = None;
            }
        }
    }
}
```

## Replanning Strategy and Triggers

### Dynamic Replanning System

```rust
#[derive(Component)]
pub struct PathfindingConfig {
    pub max_speed: f32,
    pub waypoint_tolerance: f32,
    pub slowdown_distance: f32,
    pub replanning_interval: f32,  // Seconds between replanning checks
    pub path_staleness_threshold: f32,  // Max age before forced replan
    pub obstacle_avoidance_radius: f32,
}

fn replan_invalidated_paths(
    mut agents: Query<(&Transform, &mut PathfindingAgent)>,
    nav_mesh: Res<NavigationMesh>,
    dynamic_obstacles: Query<&Transform, (With<PathfindingObstacle>, Without<PathfindingAgent>)>,
    time: Res<Time>,
) {
    for (transform, mut agent) in agents.iter_mut() {
        agent.replanning_timer += time.delta_secs();
        
        let should_replan = agent.current_path.as_ref().map_or(false, |path| {
            // Time-based replanning
            agent.replanning_timer > agent.agent_config.replanning_interval ||
            // Path age-based replanning
            time.elapsed_secs() - path.created_at > agent.agent_config.path_staleness_threshold ||
            // Obstacle-based replanning
            is_path_blocked(path, &dynamic_obstacles, agent.path_progress)
        });
        
        if should_replan {
            agent.replanning_timer = 0.0;
            // Trigger replanning in the next frame
            agent.current_path = None;
        }
    }
}

fn is_path_blocked(
    path: &NavPath, 
    obstacles: &Query<&Transform, (With<PathfindingObstacle>, Without<PathfindingAgent>)>,
    current_progress: usize,
) -> bool {
    // Check if any upcoming waypoints are blocked by dynamic obstacles
    for waypoint in path.waypoints.iter().skip(current_progress).take(3) {
        for obstacle_transform in obstacles.iter() {
            let distance = waypoint.distance(obstacle_transform.translation);
            if distance < 2.0 {  // Obstacle radius + agent radius
                return true;
            }
        }
    }
    false
}
```

### Replanning Triggers:

1. **Time-based**: Regular intervals (every 2-3 seconds)
2. **Path age**: Paths older than 10 seconds
3. **Obstacle detection**: Dynamic obstacles blocking the path
4. **Stuck detection**: Agent hasn't moved significantly in X seconds
5. **Destination changes**: New target assigned

## Code Organization for Maximum Reuse

### Shared Components and Systems

```
src/pathfinding/
├── mod.rs                  # Public API and plugin
├── components.rs           # Shared components (PathfindingAgent, etc.)
├── navigation.rs           # Core pathfinding logic
├── grid.rs                 # Navigation grid implementation
├── path_following.rs       # Path following systems
├── replanning.rs           # Dynamic replanning logic
└── integration.rs          # Integration with existing movement system
```

### Player Integration

```rust
// In src/plugins/player.rs
fn spawn_player(/* existing params */) {
    commands.spawn((
        // Existing player components...
        PathfindingAgent {
            destination: None,
            current_path: None,
            path_progress: 0,
            replanning_timer: 0.0,
            agent_config: PathfindingConfig {
                max_speed: game_config.settings.player_movement_speed,
                waypoint_tolerance: game_config.settings.player_stopping_distance,
                slowdown_distance: game_config.settings.player_slowdown_distance,
                replanning_interval: 2.0,
                path_staleness_threshold: 8.0,
                obstacle_avoidance_radius: 1.0,
            },
        },
    ));
}

fn handle_player_input(
    mut player_query: Query<&mut PathfindingAgent, With<Player>>,
    // existing params...
) {
    // When player clicks, set destination instead of direct target
    for mut agent in player_query.iter_mut() {
        if let Some(target) = ray_to_ground_target(ray.origin, *ray.direction, player_y) {
            if validate_target(player_transform.translation, target) {
                agent.destination = Some(target);
                agent.current_path = None;  // Clear existing path to trigger replanning
            }
        }
    }
}
```

### Enemy Integration

```rust
// In src/plugins/enemy.rs - replace direct movement with pathfinding
fn enemy_ai(
    mut enemy_query: Query<(&Transform, &Enemy, &mut PathfindingAgent)>,
    player_query: Query<&Transform, With<Player>>,
    // other params...
) {
    let player_pos = player_query.single().translation;
    
    for (transform, enemy, mut agent) in enemy_query.iter_mut() {
        let distance = transform.translation.distance(player_pos);
        
        if distance <= enemy.chase_distance.0 && distance > game_config.settings.enemy_stopping_distance {
            // Set pathfinding destination instead of direct movement
            agent.destination = Some(player_pos);
            
            // The pathfinding systems will handle the actual movement
        } else {
            // Stop chasing
            agent.destination = None;
            agent.current_path = None;
        }
    }
}
```

## Testing Approach

### Unit Testing Strategy

```rust
// Pure function testing
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_grid_coordinate_conversion() {
        let grid = NavGrid::new(64, 64, 2.0);
        let world_pos = Vec3::new(10.0, 0.0, 20.0);
        let grid_coords = grid.world_to_grid(world_pos).unwrap();
        let back_to_world = grid.grid_to_world(grid_coords.0, grid_coords.1);
        
        assert!((world_pos.x - back_to_world.x).abs() < 1.0);
        assert!((world_pos.z - back_to_world.z).abs() < 1.0);
    }
    
    #[test]
    fn test_pathfinding_movement_calculation() {
        let path = NavPath {
            waypoints: vec![
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(5.0, 0.0, 0.0),
                Vec3::new(10.0, 0.0, 0.0),
            ],
            created_at: 0.0,
            path_type: PathType::Direct,
        };
        
        let config = MovementConfig::default();
        let current_pos = Vec3::new(0.1, 0.0, 0.0);
        
        let (movement, new_progress) = calculate_pathfinding_movement(
            current_pos, Some(&path), 0, config
        );
        
        assert!(movement.should_move);
        assert_eq!(new_progress, 1); // Should advance to next waypoint
    }
}
```

### Integration Testing

```rust
// Component interaction testing
#[test] 
fn test_pathfinding_agent_integration() {
    let mut world = World::new();
    
    // Set up test scenario
    world.spawn((
        Transform::from_translation(Vec3::ZERO),
        PathfindingAgent::default(),
        KinematicCharacterController::default(),
    ));
    
    // Test system interactions
    // ...
}
```

### Performance Testing

```rust
#[bench]
fn bench_pathfinding_performance(b: &mut Bencher) {
    let nav_grid = create_test_grid(256, 256);
    
    b.iter(|| {
        let path = nav_grid.find_path(
            (0, 0), 
            (255, 255),
            &PathfindingConfig::default()
        );
        assert!(path.is_some());
    });
}
```

## Performance Considerations

### Optimization Strategies

1. **Hierarchical Pathfinding**: 
   - Coarse planning on reduced grid (1/4 resolution)
   - Fine planning only near agent
   - Reduces search space by ~16x

2. **Path Caching**:
   - Cache common paths between frequently visited locations
   - Use LRU eviction for memory management

3. **Async Pathfinding**:
   - Move expensive pathfinding to separate thread
   - Use channels for cross-thread communication
   - Agents use cached paths while new ones compute

4. **Spatial Partitioning**:
   - Only replan agents near dynamic obstacles
   - Use broad-phase collision detection

5. **Path Smoothing**:
   - Post-process A* paths to reduce waypoint count
   - Use string-pulling algorithm for smoother movement

### Memory Usage

- **NavGrid**: ~4 bytes per cell (64x64 = 16KB, 256x256 = 256KB)
- **Paths**: ~12 bytes per waypoint (typical path: 10-50 waypoints)
- **Agent state**: ~100 bytes per agent

### Performance Targets

- **Pathfinding**: <5ms for 64x64 grid, <20ms for 256x256 grid
- **Path following**: <0.1ms per agent per frame
- **Replanning**: <50 agents per frame without frame drops

## Implementation Priority

### Phase 1: Core System (Week 1)
1. Basic NavGrid implementation
2. A* integration with existing terrain
3. Basic PathfindingAgent component
4. Integration with existing movement system

### Phase 2: Advanced Features (Week 2)
1. Dynamic obstacle detection
2. Replanning system
3. Player and enemy integration
4. Basic testing suite

### Phase 3: Optimization (Week 3)
1. Performance optimization
2. Path smoothing
3. Advanced replanning triggers
4. Comprehensive testing

### Phase 4: Polish (Week 4)
1. Visual debugging (optional)
2. Configuration tuning
3. Documentation
4. Edge case handling

## Technical Risks and Mitigations

### Risk 1: Performance Impact
- **Mitigation**: Async pathfinding, hierarchical planning, performance budgets

### Risk 2: Integration Complexity
- **Mitigation**: Gradual integration, extensive testing, fallback to direct movement

### Risk 3: Path Quality
- **Mitigation**: Path smoothing, multiple pathfinding algorithms, tunable parameters

### Risk 4: Memory Usage
- **Mitigation**: Configurable grid resolution, path pooling, garbage collection

## Conclusion

This architecture provides a robust, testable, and performant pathfinding system that integrates seamlessly with the existing physics-based movement system. The design prioritizes code reuse between player and enemies while maintaining the flexibility to add advanced features like dynamic obstacle avoidance and path optimization.

The phased implementation approach allows for incremental development and testing, reducing risk while providing immediate value to the gameplay experience.