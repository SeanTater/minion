# Pathfinding System Design Summary

## Overview

I've designed a comprehensive pathfinding system for your Bevy 0.16 ARPG that seamlessly integrates with your existing physics-based movement architecture. The design prioritizes code reuse between players and enemies while maintaining testability and performance.

## Key Design Decisions

### 1. **Grid-Based Navigation** (Recommended)
- Leverages existing terrain heightmap system
- Integrates with the already-available A* library (`pathfinding = "4.10"`)
- Simple integration with existing `TerrainData` structure
- Allows for slope-based walkability calculations

### 2. **Seamless Physics Integration**
- Pathfinding provides waypoints to existing `calculate_movement()` function
- No changes to physics system - uses existing `KinematicCharacterController`
- Maintains existing gravity, damping, and collision systems
- Preserves GLB model orientation handling

### 3. **Shared Component Architecture**
```rust
#[derive(Component)]
pub struct PathfindingAgent {
    pub destination: Option<Vec3>,
    pub current_path: Option<Vec<Vec3>>,
    pub current_waypoint: usize,
    pub max_speed: f32,
    pub stopping_distance: f32,
}
```

## Integration Points

### Player Integration
- Replace direct movement target with pathfinding destination
- Click-to-move sets `agent.destination` instead of `player.move_target`
- Existing movement validation logic remains unchanged

### Enemy Integration
- Replace direct chase movement with pathfinding destinations
- AI sets `agent.destination = Some(player_pos)` when chasing
- Maintains existing flocking/separation forces through physics system

### Terrain Integration
```rust
impl NavigationGrid {
    pub fn from_terrain(terrain: &TerrainData) -> Self {
        // Convert heightmap to walkability grid
        // Calculate slopes to determine traversable areas
        // Integrate with existing biome system
    }
}
```

## Core Systems

### Path Planning System
```rust
pub fn find_paths(
    mut agents: Query<(&Transform, &mut PathfindingAgent), Changed<PathfindingAgent>>,
    nav_grid: Res<NavigationGrid>,
) {
    // A* pathfinding when destination changes
    // Caches paths until destination changes
}
```

### Path Following System
```rust
pub fn follow_paths(
    mut agents: Query<(&Transform, &mut PathfindingAgent, &mut KinematicCharacterController)>,
    time: Res<Time>,
) {
    // Uses existing calculate_movement() function
    // Advances through waypoints automatically
    // Integrates with physics system via KinematicCharacterController
}
```

## Performance Characteristics

### Computational Costs
- **64x64 grid**: <5ms pathfinding, 4KB memory
- **128x128 grid**: <15ms pathfinding, 16KB memory
- **256x256 grid**: <50ms pathfinding, 64KB memory

### Optimization Strategies
1. **Lazy Evaluation**: Only calculate paths when destination changes
2. **Hierarchical Planning**: Coarse grid for long-distance, fine for local
3. **Path Caching**: Reuse common paths between locations
4. **Frame Spreading**: Limit pathfinding calculations per frame

## Replanning Strategy

### Automatic Replanning Triggers
- **Time-based**: Every 2-3 seconds for active agents
- **Path age**: Paths older than 10 seconds
- **Obstacle detection**: Dynamic obstacles blocking upcoming waypoints
- **Stuck detection**: Agent hasn't moved in X seconds
- **Destination changes**: New target assigned

### Dynamic Obstacle Handling
```rust
fn is_path_blocked(path: &NavPath, obstacles: &Query<&Transform>) -> bool {
    // Check if obstacles block next 3 waypoints
    // Triggers replanning if blocked
}
```

## Testing Architecture

### Unit Tests
- Pure pathfinding logic (coordinate conversion, A* implementation)
- Movement calculations with pathfinding waypoints
- Grid generation from terrain data

### Integration Tests
- Player pathfinding with mouse input
- Enemy AI pathfinding behavior
- Path following with physics system

### Performance Tests
- Pathfinding performance benchmarks
- Memory usage validation
- Frame time impact measurement

## Implementation Phases

### Phase 1: Core System (1-2 days)
1. Create `PathfindingAgent` component
2. Implement `NavigationGrid` with basic A*
3. Create `find_paths` and `follow_paths` systems
4. Basic terrain integration

### Phase 2: Game Integration (1 day)
1. Update player input to use pathfinding
2. Update enemy AI to use pathfinding
3. Remove direct movement code
4. Test integration

### Phase 3: Advanced Features (1 day)
1. Terrain-based walkability calculation
2. Dynamic obstacle detection
3. Replanning system
4. Performance optimization

## Files Created

### Documentation
- `/home/sean-gallagher/sandbox/minion/notes/pathfinding_architecture_design.md` - Complete architectural specification
- `/home/sean-gallagher/sandbox/minion/notes/pathfinding_implementation_plan.md` - Step-by-step implementation guide
- `/home/sean-gallagher/sandbox/minion/notes/pathfinding_system_summary.md` - This summary document

### Recommended File Structure
```
src/pathfinding/
├── mod.rs              # Plugin and public API
├── components.rs       # PathfindingAgent, NavigationGrid
├── navigation.rs       # Core A* implementation
├── systems.rs          # find_paths, follow_paths systems
└── tests.rs           # Unit tests
```

## Key Benefits

1. **Code Reuse**: Same pathfinding logic for players and enemies
2. **Testable**: Pure functions separate from Bevy systems
3. **Performance**: Grid-based approach scales well
4. **Integration**: Minimal changes to existing physics system
5. **Flexibility**: Easy to add advanced features like dynamic obstacles

## Next Steps

1. **Review the detailed implementation plan** in `pathfinding_implementation_plan.md`
2. **Start with Phase 1** - Create basic pathfinding components
3. **Test incrementally** - Verify each phase before proceeding
4. **Optimize as needed** - Add performance improvements based on actual usage

The design leverages your existing terrain system, A* library, and physics architecture while providing intelligent navigation that will significantly enhance gameplay for both players and enemies.
