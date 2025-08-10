# Pathfinding System Implementation Plan

## Summary

This document provides concrete implementation steps for integrating intelligent pathfinding into the existing Minion ARPG codebase. The plan leverages the existing A* library, terrain system, and physics-based movement while minimizing complexity and maximizing code reuse.

## Quick Start Implementation

### Phase 1: Minimal Viable Pathfinding (1-2 days)

Create a simple pathfinding system that works with the existing movement logic:

#### Step 1: Create Core Components

```rust
// src/pathfinding/mod.rs
pub mod components;
pub mod navigation;
pub mod systems;

use bevy::prelude::*;
use crate::resources::GameState;

pub struct PathfindingPlugin;

impl Plugin for PathfindingPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(NavigationGrid::default())
            .add_systems(Update, (
                find_paths,
                follow_paths.after(find_paths),
            ).run_if(in_state(GameState::Playing)));
    }
}
```

#### Step 2: Define Essential Components

```rust
// src/pathfinding/components.rs
use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct PathfindingAgent {
    pub destination: Option<Vec3>,
    pub current_path: Option<Vec<Vec3>>,
    pub current_waypoint: usize,
    pub max_speed: f32,
    pub stopping_distance: f32,
}

impl Default for PathfindingAgent {
    fn default() -> Self {
        Self {
            destination: None,
            current_path: None,
            current_waypoint: 0,
            max_speed: 5.0,
            stopping_distance: 0.5,
        }
    }
}

#[derive(Resource, Debug)]
pub struct NavigationGrid {
    pub width: u32,
    pub height: u32,
    pub cell_size: f32,
    pub origin: Vec3,
    pub walkable: Vec<bool>,
}

impl Default for NavigationGrid {
    fn default() -> Self {
        Self {
            width: 64,
            height: 64,
            cell_size: 2.0,
            origin: Vec3::new(-64.0, 0.0, -64.0),
            walkable: vec![true; 64 * 64],
        }
    }
}
```

#### Step 3: Implement Core Navigation Logic

```rust
// src/pathfinding/navigation.rs
use super::components::*;
use pathfinding::prelude::astar;
use bevy::prelude::*;

impl NavigationGrid {
    pub fn world_to_grid(&self, world_pos: Vec3) -> Option<(u32, u32)> {
        let rel_x = world_pos.x - self.origin.x;
        let rel_z = world_pos.z - self.origin.z;

        if rel_x < 0.0 || rel_z < 0.0 {
            return None;
        }

        let grid_x = (rel_x / self.cell_size) as u32;
        let grid_z = (rel_z / self.cell_size) as u32;

        if grid_x >= self.width || grid_z >= self.height {
            None
        } else {
            Some((grid_x, grid_z))
        }
    }

    pub fn grid_to_world(&self, grid_x: u32, grid_z: u32) -> Vec3 {
        Vec3::new(
            self.origin.x + (grid_x as f32 + 0.5) * self.cell_size,
            0.0,
            self.origin.z + (grid_z as f32 + 0.5) * self.cell_size,
        )
    }

    pub fn is_walkable(&self, grid_x: u32, grid_z: u32) -> bool {
        if grid_x >= self.width || grid_z >= self.height {
            return false;
        }
        let index = (grid_z * self.width + grid_x) as usize;
        self.walkable.get(index).copied().unwrap_or(false)
    }

    pub fn find_path(&self, start: Vec3, end: Vec3) -> Option<Vec<Vec3>> {
        let start_grid = self.world_to_grid(start)?;
        let end_grid = self.world_to_grid(end)?;

        let result = astar(
            &start_grid,
            |&(x, z)| self.get_neighbors(x, z),
            |&(x, z)| self.heuristic((x, z), end_grid),
            |&pos| pos == end_grid,
        );

        result.map(|(path, _cost)| {
            path.into_iter()
                .map(|(x, z)| self.grid_to_world(x, z))
                .collect()
        })
    }

    fn get_neighbors(&self, x: u32, z: u32) -> Vec<((u32, u32), u32)> {
        let mut neighbors = Vec::new();
        let directions = [(-1, 0), (1, 0), (0, -1), (0, 1)];

        for (dx, dz) in directions {
            let new_x = (x as i32 + dx) as u32;
            let new_z = (z as i32 + dz) as u32;

            if self.is_walkable(new_x, new_z) {
                neighbors.push(((new_x, new_z), 10));
            }
        }

        neighbors
    }

    fn heuristic(&self, pos: (u32, u32), goal: (u32, u32)) -> u32 {
        let dx = (pos.0 as i32 - goal.0 as i32).abs() as u32;
        let dz = (pos.1 as i32 - goal.1 as i32).abs() as u32;
        (dx + dz) * 10
    }
}
```

#### Step 4: Create Pathfinding Systems

```rust
// src/pathfinding/systems.rs
use super::components::*;
use crate::game_logic::movement::{calculate_movement, MovementConfig};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub fn find_paths(
    mut agents: Query<(&Transform, &mut PathfindingAgent), Changed<PathfindingAgent>>,
    nav_grid: Res<NavigationGrid>,
) {
    for (transform, mut agent) in agents.iter_mut() {
        // Only find path if destination changed and no current path
        if let Some(destination) = agent.destination {
            if agent.current_path.is_none() {
                if let Some(path) = nav_grid.find_path(transform.translation, destination) {
                    agent.current_path = Some(path);
                    agent.current_waypoint = 0;
                    info!("Found path with {} waypoints", agent.current_path.as_ref().unwrap().len());
                } else {
                    warn!("No path found to destination");
                }
            }
        }
    }
}

pub fn follow_paths(
    mut agents: Query<(
        &Transform,
        &mut PathfindingAgent,
        &mut KinematicCharacterController
    )>,
    time: Res<Time>,
) {
    for (transform, mut agent, mut controller) in agents.iter_mut() {
        if let Some(ref path) = agent.current_path {
            if agent.current_waypoint < path.len() {
                let target_waypoint = path[agent.current_waypoint];

                let config = MovementConfig {
                    speed: agent.max_speed,
                    stopping_distance: agent.stopping_distance,
                    slowdown_distance: 2.0,
                    delta_time: time.delta_secs(),
                };

                let movement = calculate_movement(
                    transform.translation,
                    Some(target_waypoint),
                    config,
                );

                if movement.should_move {
                    // Apply movement through existing physics system
                    let movement_with_gravity = Vec3::new(
                        movement.movement_vector.x,
                        -3.0 * time.delta_secs(),
                        movement.movement_vector.z,
                    );
                    controller.translation = Some(movement_with_gravity);

                    // Handle rotation using existing logic
                    if let Some(rotation_target) = movement.rotation_target {
                        // This will be handled by the existing transform system
                        // The existing movement system already handles GLB model orientation
                    }
                } else {
                    // Reached waypoint, advance to next
                    agent.current_waypoint += 1;
                    if agent.current_waypoint >= path.len() {
                        // Reached destination
                        agent.current_path = None;
                        agent.destination = None;
                        info!("Reached destination");
                    }
                }
            }
        } else {
            // No path, stop movement
            controller.translation = Some(Vec3::new(0.0, -3.0 * time.delta_secs(), 0.0));
        }
    }
}
```

### Phase 2: Integration with Existing Systems (1 day)

#### Step 1: Update Player System

```rust
// In src/plugins/player.rs - Modify existing functions

// Add to spawn_player function:
PathfindingAgent {
    max_speed: game_config.settings.player_movement_speed,
    stopping_distance: game_config.settings.player_stopping_distance,
    ..Default::default()
},

// Modify handle_player_input function:
fn handle_player_input(
    mut player_query: Query<&mut PathfindingAgent, With<Player>>,
    // ... existing parameters
) {
    if mouse_button.just_pressed(MouseButton::Left) {
        // ... existing ray casting code

        for mut agent in player_query.iter_mut() {
            if let Some(target) = ray_to_ground_target(ray.origin, *ray.direction, player_y) {
                if validate_target(player_transform.translation, target) {
                    agent.destination = Some(target);
                    agent.current_path = None; // Clear existing path
                    info!("Player pathfinding target set: ({:.2}, {:.2}, {:.2})",
                          target.x, target.y, target.z);
                }
            }
        }
    }
}

// Remove or comment out the existing move_player system since pathfinding handles it
```

#### Step 2: Update Enemy System

```rust
// In src/plugins/enemy.rs - Modify existing functions

// Add to spawn_enemies (in the spawn command):
PathfindingAgent {
    max_speed: enemy.speed.0 * game_config.settings.enemy_speed_multiplier,
    stopping_distance: game_config.settings.enemy_stopping_distance,
    ..Default::default()
},

// Modify enemy_ai function:
fn enemy_ai(
    mut enemy_query: Query<(&Transform, &Enemy, &mut PathfindingAgent)>,
    player_query: Query<&Transform, With<Player>>,
    // ... other existing parameters
) {
    let player_pos = player_query.single().translation;

    for (transform, enemy, mut agent) in enemy_query.iter_mut() {
        if enemy.is_dying {
            continue;
        }

        let distance = transform.translation.distance(player_pos);

        if distance <= enemy.chase_distance.0 && distance > agent.stopping_distance {
            // Set pathfinding destination instead of direct movement
            agent.destination = Some(player_pos);
        } else {
            // Stop chasing
            agent.destination = None;
            agent.current_path = None;
        }
    }
}

// Remove or comment out the kinematic movement code in enemy_ai since pathfinding handles it
```

#### Step 3: Initialize Navigation Grid from Terrain

```rust
// In src/plugins/map_loader.rs - Add after terrain loading

use crate::pathfinding::NavigationGrid;

fn initialize_navigation_grid(
    mut commands: Commands,
    map: Res<MapDefinition>,
) {
    let terrain = &map.terrain;
    let nav_grid = NavigationGrid::from_terrain(terrain);
    commands.insert_resource(nav_grid);
}

// Add to MapLoaderPlugin:
.add_systems(OnEnter(GameState::Playing),
    initialize_navigation_grid.after(load_map))
```

### Phase 3: Terrain Integration (1 day)

#### Step 1: Enhance NavigationGrid with Terrain Data

```rust
// Add to NavigationGrid implementation
impl NavigationGrid {
    pub fn from_terrain(terrain: &TerrainData) -> Self {
        let width = terrain.width;
        let height = terrain.height;
        let cell_size = terrain.scale;

        // Center the grid on the terrain
        let world_width = width as f32 * cell_size;
        let world_height = height as f32 * cell_size;
        let origin = Vec3::new(-world_width / 2.0, 0.0, -world_height / 2.0);

        let mut walkable = Vec::with_capacity((width * height) as usize);

        for z in 0..height {
            for x in 0..width {
                // Calculate slope to determine walkability
                let current_height = terrain.get_height_at_grid(x, z).unwrap_or(0.0);

                // Check neighboring cells for slope calculation
                let mut max_slope = 0.0;
                for (dx, dz) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                    let nx = (x as i32 + dx) as u32;
                    let nz = (z as i32 + dz) as u32;

                    if let Some(neighbor_height) = terrain.get_height_at_grid(nx, nz) {
                        let height_diff = (current_height - neighbor_height).abs();
                        let slope = height_diff / cell_size;
                        max_slope = max_slope.max(slope);
                    }
                }

                // Walkable if slope is reasonable (less than 45 degrees = slope of 1.0)
                let is_walkable = max_slope < 0.8; // Slightly less than 45 degrees for safety
                walkable.push(is_walkable);
            }
        }

        Self {
            width,
            height,
            cell_size,
            origin,
            walkable,
        }
    }
}
```

## Testing Strategy

### Unit Tests

```rust
// src/pathfinding/tests.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_coordinate_conversion() {
        let grid = NavigationGrid {
            width: 10,
            height: 10,
            cell_size: 2.0,
            origin: Vec3::new(-10.0, 0.0, -10.0),
            walkable: vec![true; 100],
        };

        let world_pos = Vec3::new(0.0, 0.0, 0.0);
        let grid_coords = grid.world_to_grid(world_pos).unwrap();
        assert_eq!(grid_coords, (5, 5));

        let back_to_world = grid.grid_to_world(5, 5);
        assert!((world_pos.x - back_to_world.x).abs() < 0.1);
        assert!((world_pos.z - back_to_world.z).abs() < 0.1);
    }

    #[test]
    fn test_simple_pathfinding() {
        let mut grid = NavigationGrid {
            width: 5,
            height: 5,
            cell_size: 1.0,
            origin: Vec3::new(-2.5, 0.0, -2.5),
            walkable: vec![true; 25],
        };

        let start = Vec3::new(-2.0, 0.0, -2.0);
        let end = Vec3::new(2.0, 0.0, 2.0);

        let path = grid.find_path(start, end);
        assert!(path.is_some());
        assert!(path.unwrap().len() > 1);
    }
}
```

### Integration Testing

Create a simple test scene to verify pathfinding works:

```rust
// src/pathfinding/integration_tests.rs
use bevy::prelude::*;

fn test_pathfinding_scene() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(PathfindingPlugin)
       .insert_resource(NavigationGrid::default());

    // Spawn test agent
    app.world.spawn((
        Transform::from_translation(Vec3::new(-10.0, 0.0, -10.0)),
        PathfindingAgent {
            destination: Some(Vec3::new(10.0, 0.0, 10.0)),
            ..Default::default()
        },
    ));

    app
}
```

## Performance Considerations

### Optimization Checklist

1. **Lazy Path Finding**: Only calculate paths when destination changes
2. **Path Caching**: Cache common paths between locations
3. **Limited Search Depth**: Set maximum search distance to prevent expensive long-range pathfinding
4. **Hierarchical Planning**: Use coarse grid for long-distance, fine grid for local navigation
5. **Frame Spreading**: Limit pathfinding calculations per frame

### Memory Usage

- **64x64 grid**: ~4KB for walkability data
- **128x128 grid**: ~16KB for walkability data
- **256x256 grid**: ~64KB for walkability data

Choose grid resolution based on world size and detail requirements.

## File Structure

```
src/pathfinding/
├── mod.rs              # Plugin and public API
├── components.rs       # PathfindingAgent, NavigationGrid
├── navigation.rs       # Core pathfinding algorithms
├── systems.rs          # Bevy systems (find_paths, follow_paths)
└── tests.rs           # Unit tests
```

## Quick Start Commands

1. **Create the pathfinding module**:
   ```bash
   mkdir src/pathfinding
   touch src/pathfinding/{mod.rs,components.rs,navigation.rs,systems.rs}
   ```

2. **Add to src/lib.rs**:
   ```rust
   pub mod pathfinding;
   ```

3. **Add to main.rs**:
   ```rust
   use minion::pathfinding::PathfindingPlugin;

   app.add_plugins(PathfindingPlugin);
   ```

4. **Test the implementation**:
   ```bash
   cargo test pathfinding
   cargo run
   ```

## Next Steps

1. **Start with Phase 1**: Implement basic pathfinding components and systems
2. **Test Integration**: Verify player and enemy pathfinding works with existing movement
3. **Add Terrain Integration**: Connect navigation grid to terrain heightmaps
4. **Optimize Performance**: Add path caching and hierarchical planning as needed
5. **Add Advanced Features**: Dynamic obstacle avoidance, path smoothing, visual debugging

This implementation plan provides a practical, step-by-step approach to adding intelligent pathfinding while maintaining compatibility with your existing physics-based movement system.
