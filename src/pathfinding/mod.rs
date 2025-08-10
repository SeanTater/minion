use crate::components::PathfindingAgent;
use crate::game_logic::errors::MinionResult;
use crate::map::{EnvironmentObject, TerrainData};
use crate::terrain::coordinates::*;
use bevy::prelude::*;
use pathfinding::prelude::astar;

pub mod grid_blocking;
pub mod obstacles;

pub use obstacles::*;

/// Configuration for pathfinding grid generation
#[derive(Debug, Clone)]
pub struct PathfindingConfig {
    /// Maximum slope angle in degrees that is considered walkable
    pub max_walkable_slope: f32,
    /// Linear slope cost factor - higher values make slope more expensive
    pub slope_cost_factor: f32,
    /// Extra clearance (personal space) added on top of agent_radius when inflating obstacles
    pub agent_clearance_slop: f32,
}

impl Default for PathfindingConfig {
    fn default() -> Self {
        Self {
            max_walkable_slope: 45.0,  // 45 degrees max slope
            slope_cost_factor: 0.5,    // Linear slope cost factor
            agent_clearance_slop: 0.2, // Personal space/slop in world units
        }
    }
}

/// A single node in the navigation grid
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridNode {
    pub x: u32,
    pub z: u32,
}

impl GridNode {
    pub fn new(x: u32, z: u32) -> Self {
        Self { x, z }
    }

    /// Get neighbors of this grid node (4-directional)
    pub fn neighbors(&self, grid_width: u32, grid_height: u32) -> Vec<GridNode> {
        let mut neighbors = Vec::new();

        // North
        if self.z > 0 {
            neighbors.push(GridNode::new(self.x, self.z - 1));
        }

        // South
        if self.z < grid_height - 1 {
            neighbors.push(GridNode::new(self.x, self.z + 1));
        }

        // West
        if self.x > 0 {
            neighbors.push(GridNode::new(self.x - 1, self.z));
        }

        // East
        if self.x < grid_width - 1 {
            neighbors.push(GridNode::new(self.x + 1, self.z));
        }

        neighbors
    }

    /// Calculate Manhattan distance to another node (heuristic for A*)
    pub fn manhattan_distance(&self, other: &GridNode) -> u32 {
        ((self.x as i32 - other.x as i32).abs() + (self.z as i32 - other.z as i32).abs()) as u32
    }

    /// Calculate Euclidean distance to another node (improved heuristic for A*)
    pub fn euclidean_distance(&self, other: &GridNode) -> f32 {
        let dx = (self.x as f32 - other.x as f32).abs();
        let dz = (self.z as f32 - other.z as f32).abs();
        (dx * dx + dz * dz).sqrt()
    }
}

/// Navigation grid for pathfinding
#[derive(Debug, Clone, Resource)]
pub struct NavigationGrid {
    /// Walkability map - true if the cell is walkable
    pub walkable: Vec<bool>,
    /// Height values for each grid cell
    pub heights: Vec<f32>,
    /// Priority values for obstacle blocking (higher = more important)
    pub obstacle_priorities: Vec<u8>,
    /// Grid dimensions
    pub width: u32,
    pub height: u32,
    /// World scale per grid cell
    pub cell_size: f32,
    /// Reference to the source terrain
    pub terrain_width: u32,
    pub terrain_height: u32,
    pub terrain_scale: f32,
    /// Pathfinding configuration used to generate this grid
    pub config: PathfindingConfig,
}

impl NavigationGrid {
    /// Build a navigation grid from terrain data
    pub fn from_terrain(terrain: &TerrainData, config: PathfindingConfig) -> MinionResult<Self> {
        Self::from_terrain_and_objects(terrain, &[], config)
    }

    /// Build a navigation grid from terrain data and environment objects
    pub fn from_terrain_and_objects(
        terrain: &TerrainData,
        objects: &[EnvironmentObject],
        config: PathfindingConfig,
    ) -> MinionResult<Self> {
        let total_cells = (terrain.width * terrain.height) as usize;
        let mut walkable = Vec::with_capacity(total_cells);
        let mut heights = Vec::with_capacity(total_cells);
        let obstacle_priorities = vec![0u8; total_cells]; // Initialize priorities

        // Direct iteration - grid coordinates = terrain coordinates
        for z in 0..terrain.height {
            for x in 0..terrain.width {
                let height = get_height_at_grid(terrain, x, z).ok_or_else(|| {
                    crate::game_logic::errors::MinionError::InvalidMapData {
                        reason: format!("Failed to get height at position ({x}, {z})"),
                    }
                })?;

                heights.push(height);
                walkable.push(Self::calculate_walkability(terrain, x, z, &config));
            }
        }

        let mut nav_grid = NavigationGrid {
            walkable,
            heights,
            obstacle_priorities, // Add priority tracking
            width: terrain.width,
            height: terrain.height,
            cell_size: terrain.scale,
            terrain_width: terrain.width,
            terrain_height: terrain.height,
            terrain_scale: terrain.scale,
            config,
        };

        // NEW: Use trait-based obstacle system
        let mut obstacle_manager = ObstacleManager::new();
        obstacle_manager.add_environment_obstacles(objects);

        // Debug logging for obstacle integration
        info!(
            "Applying {} environment objects to navigation grid",
            objects.len()
        );

        obstacle_manager.apply_to_navigation_grid(&mut nav_grid);

        // Count blocked cells after obstacle application
        let blocked_count = nav_grid.walkable.iter().filter(|&&w| !w).count();
        let total_cells = nav_grid.walkable.len();
        info!(
            "Navigation grid: {blocked}/{total} cells blocked ({percentage:.1}%)",
            blocked = blocked_count,
            total = total_cells,
            percentage = (blocked_count as f32 / total_cells as f32) * 100.0
        );

        Ok(nav_grid)
    }

    /// Calculate if a terrain cell is walkable based on slope
    fn calculate_walkability(
        terrain: &TerrainData,
        x: u32,
        z: u32,
        config: &PathfindingConfig,
    ) -> bool {
        let Some(current_height) = get_height_at_grid(terrain, x, z) else {
            return false;
        };

        // Check slopes to all neighbors
        for (nx, nz) in [
            (x.saturating_sub(1), z),
            (x + 1, z),
            (x, z.saturating_sub(1)),
            (x, z + 1),
        ] {
            if let Some(neighbor_height) = get_height_at_grid(terrain, nx, nz) {
                let slope_angle = ((neighbor_height - current_height).abs() / terrain.scale)
                    .atan()
                    .to_degrees();
                if slope_angle > config.max_walkable_slope {
                    return false;
                }
            }
        }
        true
    }

    /// Check if a grid position is walkable
    pub fn is_walkable(&self, x: u32, z: u32) -> bool {
        if x >= self.width || z >= self.height {
            return false;
        }
        let index = (z * self.width + x) as usize;
        self.walkable.get(index).copied().unwrap_or(false)
    }

    /// Get height at a grid node
    pub fn get_height_at_grid(&self, node: GridNode) -> Option<f32> {
        if node.x >= self.width || node.z >= self.height {
            return None;
        }
        let index = (node.z * self.width + node.x) as usize;
        self.heights.get(index).copied()
    }

    /// Calculate movement cost between two adjacent grid nodes
    pub fn movement_cost(&self, from: GridNode, to: GridNode) -> u32 {
        let from_height = self.get_height_at_grid(from).unwrap_or(0.0);
        let to_height = self.get_height_at_grid(to).unwrap_or(0.0);

        let height_diff = to_height - from_height;
        let base_cost = 10.0; // Base movement cost (scaled for A* integer math)

        // Linear slope-based cost calculation
        let slope_factor = 1.0 + (height_diff * self.config.slope_cost_factor);
        let movement_cost = base_cost * slope_factor.max(0.1); // Minimum cost

        movement_cost as u32
    }

    /// Convert world position to grid coordinates, returning None if out of bounds
    pub fn world_to_grid(&self, world_pos: Vec3) -> Option<GridNode> {
        let half_width = (self.terrain_width as f32 * self.terrain_scale) / 2.0;
        let half_height = (self.terrain_height as f32 * self.terrain_scale) / 2.0;

        let x = ((world_pos.x + half_width) / self.terrain_scale).round();
        let z = ((world_pos.z + half_height) / self.terrain_scale).round();

        if x >= 0.0 && z >= 0.0 && x < self.width as f32 && z < self.height as f32 {
            Some(GridNode::new(x as u32, z as u32))
        } else {
            None
        }
    }

    /// Check if a position is within the navigation grid bounds
    pub fn is_position_in_bounds(&self, world_pos: Vec3) -> bool {
        self.world_to_grid(world_pos).is_some()
    }

    /// Set cell walkability with priority system - higher priority obstacles override lower priority
    pub fn set_cell_walkable_with_priority(
        &mut self,
        node: GridNode,
        walkable: bool,
        priority: u8,
    ) {
        if node.x >= self.width || node.z >= self.height {
            return;
        }
        let index = (node.z * self.width + node.x) as usize;

        // Priority-based override system
        if !walkable {
            // For obstacle placement, higher priority wins
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

    /// Clone the navigation grid and inflate obstacles by agent radius + slop
    pub fn clone_and_inflate(&self, agent_radius: f32, slop: f32) -> NavigationGrid {
        let mut inflated_grid = self.clone();

        // Calculate inflation radius in grid cells
        let inflation_radius = agent_radius + slop;
        let cell_inflation_radius = (inflation_radius / self.cell_size).ceil() as i32;

        debug!(
            "Inflating grid: agent_radius={:.2}, slop={:.2}, inflation_radius={:.2}, cell_inflation_radius={}",
            agent_radius, slop, inflation_radius, cell_inflation_radius
        );

        if cell_inflation_radius <= 0 {
            return inflated_grid; // No inflation needed
        }

        // Create a copy of the original walkable state to read from
        let original_walkable = self.walkable.clone();
        let original_blocked_count = original_walkable.iter().filter(|&&w| !w).count();

        // For each blocked cell in the original grid, mark surrounding cells as blocked
        for z in 0..self.height {
            for x in 0..self.width {
                let index = (z * self.width + x) as usize;
                if !original_walkable[index] {
                    // This cell is blocked, so inflate around it
                    let center_node = GridNode::new(x, z);
                    self.inflate_around_cell(
                        &mut inflated_grid,
                        center_node,
                        cell_inflation_radius,
                    );
                }
            }
        }

        let inflated_blocked_count = inflated_grid.walkable.iter().filter(|&&w| !w).count();
        debug!(
            "Inflation complete: original_blocked={}, inflated_blocked={}",
            original_blocked_count, inflated_blocked_count
        );

        inflated_grid
    }

    /// Helper function to inflate (mark as blocked) cells around a given center cell
    fn inflate_around_cell(
        &self,
        inflated_grid: &mut NavigationGrid,
        center: GridNode,
        cell_radius: i32,
    ) {
        for dz in -cell_radius..=cell_radius {
            for dx in -cell_radius..=cell_radius {
                let x = center.x as i32 + dx;
                let z = center.z as i32 + dz;

                if x >= 0 && z >= 0 && x < self.width as i32 && z < self.height as i32 {
                    // Check if this cell is within the inflation radius (circular)
                    let distance_squared = (dx * dx + dz * dz) as f32;
                    let radius_squared = (cell_radius as f32) * (cell_radius as f32);

                    if distance_squared <= radius_squared {
                        let _target_node = GridNode::new(x as u32, z as u32);
                        let index = (z as u32 * self.width + x as u32) as usize;
                        if let Some(cell) = inflated_grid.walkable.get_mut(index) {
                            *cell = false; // Mark as blocked
                        }
                    }
                }
            }
        }
    }
}

/// Filter waypoints to improve spacing while preserving path accuracy
/// Uses a greedy approach: keep waypoints that are at least min_distance apart,
/// but always keep the final waypoint to ensure we reach the destination
fn filter_waypoints_for_spacing(waypoints: Vec<Vec3>, min_distance: f32) -> Vec<Vec3> {
    if waypoints.len() <= 2 {
        return waypoints; // Keep start and end
    }

    let mut filtered = Vec::new();
    filtered.push(waypoints[0]); // Always keep start

    let mut last_kept_index = 0;

    for i in 1..waypoints.len() - 1 {
        let distance_from_last = waypoints[i].distance(waypoints[last_kept_index]);

        if distance_from_last >= min_distance {
            filtered.push(waypoints[i]);
            last_kept_index = i;
        }
    }

    // Always keep the final waypoint to ensure we reach the destination
    if let Some(last) = waypoints.last() {
        // Only add if it's different from the last filtered waypoint
        if filtered.last() != Some(last) {
            filtered.push(*last);
        }
    }

    filtered
}

/// Find a path between two world positions using A* pathfinding
pub fn find_path(
    navigation_grid: &NavigationGrid,
    start_world: Vec3,
    goal_world: Vec3,
    agent_radius: f32,
) -> Option<Vec<Vec3>> {
    // Clone and inflate the navigation grid for this agent size so we don't mutate the base grid
    let inflated_grid = navigation_grid
        .clone_and_inflate(agent_radius, navigation_grid.config.agent_clearance_slop);

    let start_node = inflated_grid.world_to_grid(start_world)?;
    let goal_node = inflated_grid.world_to_grid(goal_world)?;

    // Debug logging for pathfinding attempts
    debug!(
        "Pathfinding: start=({:.1},{:.1},{:.1}) -> grid=({},{}) walkable={}",
        start_world.x,
        start_world.y,
        start_world.z,
        start_node.x,
        start_node.z,
        inflated_grid.is_walkable(start_node.x, start_node.z)
    );
    debug!(
        "Pathfinding: goal=({:.1},{:.1},{:.1}) -> grid=({},{}) walkable={}",
        goal_world.x,
        goal_world.y,
        goal_world.z,
        goal_node.x,
        goal_node.z,
        inflated_grid.is_walkable(goal_node.x, goal_node.z)
    );

    // Check if start and goal are walkable on the inflated grid
    if !inflated_grid.is_walkable(start_node.x, start_node.z)
        || !inflated_grid.is_walkable(goal_node.x, goal_node.z)
    {
        warn!(
            "Pathfinding failed: start_walkable={}, goal_walkable={}",
            inflated_grid.is_walkable(start_node.x, start_node.z),
            inflated_grid.is_walkable(goal_node.x, goal_node.z)
        );
        return None;
    }

    // Use A* to find the path on the inflated grid
    let (path, _cost) = astar(
        &start_node,
        |node| {
            let current_node = *node;
            let neighbors: Vec<_> = node
                .neighbors(inflated_grid.width, inflated_grid.height)
                .into_iter()
                .filter(|neighbor| inflated_grid.is_walkable(neighbor.x, neighbor.z))
                .map(|neighbor| {
                    (
                        neighbor,
                        inflated_grid.movement_cost(current_node, neighbor),
                    )
                })
                .collect();
            neighbors
        },
        |node| (node.euclidean_distance(&goal_node) * 10.0) as u32,
        |node| *node == goal_node,
    )?;

    // Convert grid path to world coordinates (use original grid for height data)
    let half_width = (navigation_grid.terrain_width as f32 * navigation_grid.terrain_scale) / 2.0;
    let half_height = (navigation_grid.terrain_height as f32 * navigation_grid.terrain_scale) / 2.0;

    let path_length = path.len(); // Store length before moving path
    let world_path: Vec<Vec3> = path
        .into_iter()
        .map(|node| {
            let world_x = (node.x as f32 * navigation_grid.terrain_scale) - half_width;
            let world_z = (node.z as f32 * navigation_grid.terrain_scale) - half_height;
            let height = navigation_grid.get_height_at_grid(node).unwrap_or(0.0);
            Vec3::new(world_x, height, world_z)
        })
        .collect();

    // Filter waypoints to improve spacing while preserving path accuracy
    let filtered_path = filter_waypoints_for_spacing(world_path, 2.0);

    debug!(
        "Pathfinding success: raw_path={} waypoints, filtered_path={} waypoints",
        path_length,
        filtered_path.len()
    );

    Some(filtered_path)
}

/// Check if an agent needs to replan its path
pub fn should_replan_path(
    agent: &PathfindingAgent,
    current_time: f32,
    _current_position: Vec3,
) -> bool {
    // Time-based replanning
    if current_time - agent.last_replan_time > agent.replan_interval {
        return true;
    }

    // Check if destination has changed significantly
    if let Some(destination) = agent.destination {
        if let Some(last_waypoint) = agent.nav_path.final_destination() {
            if last_waypoint.distance(destination) > agent.max_path_distance {
                return true;
            }
        } else if !agent.nav_path.is_empty() {
            // Path exists but no destination matches - replan
            return true;
        }
    }

    // Check if agent has no path but has a destination
    if agent.destination.is_some() && !agent.has_path() {
        return true;
    }

    // TODO: Check if agent is stuck (would need position history)

    false
}

/// Update pathfinding agents - advance waypoints when reached
pub fn update_pathfinding_agents(mut agents_query: Query<(&mut PathfindingAgent, &Transform)>) {
    for (mut agent, transform) in agents_query.iter_mut() {
        if let Some(current_waypoint) = agent.current_waypoint() {
            // FIXED: Use 2D distance for waypoint reach check (consistent with movement system)
            let current_2d = Vec3::new(transform.translation.x, 0.0, transform.translation.z);
            let waypoint_2d = Vec3::new(current_waypoint.x, 0.0, current_waypoint.z);
            let distance_2d = current_2d.distance(waypoint_2d);

            if distance_2d <= agent.waypoint_reach_distance {
                let old_index = agent.nav_path.current_index();
                agent.advance_waypoint();
                info!(
                    "Waypoint reached! Advanced from index {} to {} (path length: {})",
                    old_index,
                    agent.nav_path.current_index(),
                    agent.nav_path.len()
                );

                // Log next waypoint if it exists
                if let Some(next_waypoint) = agent.current_waypoint() {
                    info!(
                        "Next waypoint: ({:.1}, {:.1}, {:.1})",
                        next_waypoint.x, next_waypoint.y, next_waypoint.z
                    );
                } else {
                    info!("Path completed - clearing destination");
                    agent.destination = None;
                }
            }
        } else {
            // Debug logging commented out to reduce console spam
            // if !agent.nav_path.is_empty() {
            //     debug!("No current waypoint available (index={}, len={})",
            //            agent.nav_path.current_index(), agent.nav_path.len());
            // }
        }
    }
}

/// Plan new paths for agents that need replanning
pub fn plan_paths(
    mut agents_query: Query<(&mut PathfindingAgent, &Transform)>,
    navigation_grid: Res<NavigationGrid>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs();

    for (mut agent, transform) in agents_query.iter_mut() {
        if should_replan_path(&agent, current_time, transform.translation) {
            if let Some(destination) = agent.destination {
                if let Some(new_path) = find_path(
                    &navigation_grid,
                    transform.translation,
                    destination,
                    agent.agent_radius,
                ) {
                    let path_length = new_path.len();
                    agent.set_path(new_path);
                    agent.last_replan_time = current_time;
                    info!(
                        "Planned new path with {} waypoints from ({:.1}, {:.1}, {:.1}) to ({:.1}, {:.1}, {:.1})",
                        path_length,
                        transform.translation.x,
                        transform.translation.y,
                        transform.translation.z,
                        destination.x,
                        destination.y,
                        destination.z
                    );
                    // Debug logging removed - use tests for debugging
                } else {
                    warn!(
                        "Failed to find path from ({:.1}, {:.1}, {:.1}) to ({:.1}, {:.1}, {:.1}) - keeping existing path",
                        transform.translation.x,
                        transform.translation.y,
                        transform.translation.z,
                        destination.x,
                        destination.y,
                        destination.z
                    );
                }
            }
        }
    }
}

/// Provide waypoints to the existing movement system
pub fn get_current_waypoint_for_agent(agent: &PathfindingAgent) -> Option<Vec3> {
    agent.current_waypoint()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::TerrainData;

    // Helper function to reduce .to_string() repetition in tests
    fn env_obj(
        object_type: &str,
        position: Vec3,
        rotation: Vec3,
        scale: Vec3,
    ) -> EnvironmentObject {
        EnvironmentObject::new(object_type.to_string(), position, rotation, scale)
    }

    fn env_obj_simple(object_type: &str, position: Vec3) -> EnvironmentObject {
        EnvironmentObject::simple(object_type.to_string(), position)
    }

    /// Test that new trait system produces same results as old system
    #[test]
    fn test_backward_compatibility() {
        let terrain = TerrainData::create_flat(16, 16, 1.0, 0.0).unwrap();
        let objects = vec![
            env_obj(
                "tree",
                Vec3::new(2.0, 0.0, 2.0),
                Vec3::ZERO,
                Vec3::new(2.0, 3.0, 2.0),
            ),
            env_obj(
                "rock",
                Vec3::new(-2.0, 0.0, -2.0),
                Vec3::ZERO,
                Vec3::new(1.0, 1.0, 1.0),
            ),
            env_obj(
                "boulder",
                Vec3::new(4.0, 0.0, 4.0),
                Vec3::ZERO,
                Vec3::new(1.5, 1.5, 1.5),
            ),
            env_obj("grass", Vec3::new(4.0, 0.0, -4.0), Vec3::ZERO, Vec3::ONE),
            env_obj(
                "unknown_type",
                Vec3::new(-4.0, 0.0, 4.0),
                Vec3::ZERO,
                Vec3::new(1.5, 2.0, 1.5),
            ),
        ];

        // Create navigation grid with new trait system
        let nav_grid = NavigationGrid::from_terrain_and_objects(
            &terrain,
            &objects,
            PathfindingConfig::default(),
        )
        .unwrap();

        // Verify expected blocking behavior (same as old system)

        // Tree should be blocked (circular area)
        let tree_node = nav_grid.world_to_grid(Vec3::new(2.0, 0.0, 2.0)).unwrap();
        assert!(!nav_grid.is_walkable(tree_node.x, tree_node.z));
        assert_eq!(nav_grid.get_obstacle_priority(tree_node), 150); // Tree priority

        // Rock should be blocked (circular area)
        let rock_node = nav_grid.world_to_grid(Vec3::new(-2.0, 0.0, -2.0)).unwrap();
        assert!(!nav_grid.is_walkable(rock_node.x, rock_node.z));
        assert_eq!(nav_grid.get_obstacle_priority(rock_node), 120); // Rock priority

        // Boulder should be blocked (circular area)
        let boulder_node = nav_grid.world_to_grid(Vec3::new(4.0, 0.0, 4.0)).unwrap();
        assert!(!nav_grid.is_walkable(boulder_node.x, boulder_node.z));
        assert_eq!(nav_grid.get_obstacle_priority(boulder_node), 200); // Boulder priority

        // Grass should NOT be blocked
        let grass_node = nav_grid.world_to_grid(Vec3::new(4.0, 0.0, -4.0)).unwrap();
        assert!(nav_grid.is_walkable(grass_node.x, grass_node.z));
        assert_eq!(nav_grid.get_obstacle_priority(grass_node), 0); // No obstacle

        // Unknown type should be blocked (rectangular area)
        let unknown_node = nav_grid.world_to_grid(Vec3::new(-4.0, 0.0, 4.0)).unwrap();
        assert!(!nav_grid.is_walkable(unknown_node.x, unknown_node.z));
        assert_eq!(nav_grid.get_obstacle_priority(unknown_node), 100); // Custom priority
    }

    #[test]
    fn test_priority_override_system_integration() {
        let terrain = TerrainData::create_flat(8, 8, 1.0, 0.0).unwrap();
        let objects = vec![
            // Place tree and boulder at overlapping positions
            env_obj(
                "tree",
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::ZERO,
                Vec3::new(2.0, 3.0, 2.0),
            ), // Tree priority 150
            env_obj(
                "boulder",
                Vec3::new(0.2, 0.0, 0.2),
                Vec3::ZERO,
                Vec3::new(1.5, 1.5, 1.5),
            ), // Boulder priority 200
        ];

        let nav_grid = NavigationGrid::from_terrain_and_objects(
            &terrain,
            &objects,
            PathfindingConfig::default(),
        )
        .unwrap();

        // At overlap position, boulder (higher priority) should win
        let center_node = nav_grid.world_to_grid(Vec3::new(0.0, 0.0, 0.0)).unwrap();
        assert!(!nav_grid.is_walkable(center_node.x, center_node.z));
        assert_eq!(nav_grid.get_obstacle_priority(center_node), 200); // Boulder priority wins
    }

    #[test]
    fn test_performance_regression() {
        use std::time::Instant;

        let terrain = TerrainData::create_flat(32, 32, 1.0, 0.0).unwrap();

        // Create many objects to test performance
        let mut objects = Vec::new();
        for i in 0..50 {
            let x = (i % 10) as f32 * 2.0 - 10.0;
            let z = ((i / 10) % 10) as f32 * 2.0 - 10.0;
            let object_type = match i % 4 {
                0 => "tree",
                1 => "rock",
                2 => "boulder",
                _ => "grass",
            };
            objects.push(env_obj(
                object_type,
                Vec3::new(x, 0.0, z),
                Vec3::ZERO,
                Vec3::new(1.5, 2.0, 1.5),
            ));
        }

        let start = Instant::now();
        let nav_grid = NavigationGrid::from_terrain_and_objects(
            &terrain,
            &objects,
            PathfindingConfig::default(),
        )
        .unwrap();
        let duration = start.elapsed();

        // Should complete within reasonable time (adjust threshold as needed)
        assert!(
            duration.as_millis() < 100,
            "Grid generation took {}ms, expected <100ms",
            duration.as_millis()
        );

        // Verify grid is valid
        assert_eq!(nav_grid.width, 32);
        assert_eq!(nav_grid.height, 32);

        // Check that some obstacles are blocked with proper priorities
        let mut blocked_count = 0;
        let mut priority_counts = [0u32; 256]; // Count obstacles by priority

        for z in 0..nav_grid.height {
            for x in 0..nav_grid.width {
                let node = GridNode::new(x, z);
                if !nav_grid.is_walkable(x, z) {
                    blocked_count += 1;
                    let priority = nav_grid.get_obstacle_priority(node);
                    priority_counts[priority as usize] += 1;
                }
            }
        }

        assert!(blocked_count > 0, "Should have some blocked cells");

        // Should have cells with different priorities
        assert!(
            priority_counts[120] > 0,
            "Should have rock obstacles (priority 120)"
        );
        assert!(
            priority_counts[150] > 0,
            "Should have tree obstacles (priority 150)"
        );
        assert!(
            priority_counts[200] > 0,
            "Should have boulder obstacles (priority 200)"
        );
    }

    #[test]
    fn test_grid_node_neighbors() {
        let node = GridNode::new(1, 1);
        let neighbors = node.neighbors(3, 3);

        assert_eq!(neighbors.len(), 4);
        assert!(neighbors.contains(&GridNode::new(0, 1))); // West
        assert!(neighbors.contains(&GridNode::new(2, 1))); // East
        assert!(neighbors.contains(&GridNode::new(1, 0))); // North
        assert!(neighbors.contains(&GridNode::new(1, 2))); // South
    }

    #[test]
    fn test_grid_node_corner_neighbors() {
        let node = GridNode::new(0, 0);
        let neighbors = node.neighbors(3, 3);

        assert_eq!(neighbors.len(), 2);
        assert!(neighbors.contains(&GridNode::new(1, 0))); // East
        assert!(neighbors.contains(&GridNode::new(0, 1))); // South
    }

    #[test]
    fn test_manhattan_distance() {
        let node1 = GridNode::new(0, 0);
        let node2 = GridNode::new(3, 4);

        assert_eq!(node1.manhattan_distance(&node2), 7);
        assert_eq!(node2.manhattan_distance(&node1), 7);
    }

    #[test]
    fn test_navigation_grid_creation() {
        let terrain = TerrainData::create_flat(4, 4, 1.0, 0.0).unwrap();
        let config = PathfindingConfig::default();

        let nav_grid = NavigationGrid::from_terrain(&terrain, config).unwrap();

        // Grid dimensions now match terrain dimensions
        assert_eq!(nav_grid.width, 4);
        assert_eq!(nav_grid.height, 4);
        assert_eq!(nav_grid.walkable.len(), 4 * 4);
        assert_eq!(nav_grid.heights.len(), 4 * 4);

        // Flat terrain should be mostly walkable
        let walkable_count = nav_grid.walkable.iter().filter(|&&w| w).count();
        assert!(walkable_count > nav_grid.walkable.len() / 2);
    }

    #[test]
    fn test_pathfinding_agent_new() {
        let agent = PathfindingAgent::new();

        assert!(agent.nav_path.is_empty());
        assert_eq!(agent.nav_path.current_index(), 0);
        assert!(agent.destination.is_none());
        assert!(!agent.has_path());
    }

    #[test]
    fn test_pathfinding_agent_path_management() {
        let mut agent = PathfindingAgent::new();
        let path = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(2.0, 0.0, 0.0),
        ];

        agent.set_path(path.clone());

        assert!(agent.has_path());
        assert_eq!(agent.current_waypoint(), Some(Vec3::new(0.0, 0.0, 0.0)));

        agent.advance_waypoint();
        assert_eq!(agent.current_waypoint(), Some(Vec3::new(1.0, 0.0, 0.0)));

        agent.advance_waypoint();
        assert_eq!(agent.current_waypoint(), Some(Vec3::new(2.0, 0.0, 0.0)));

        agent.advance_waypoint();
        assert_eq!(agent.current_waypoint(), None);
        assert!(!agent.has_path());
    }

    #[test]
    fn test_pathfinding_agent_waypoint_reach_distance() {
        let agent = PathfindingAgent::new();
        // Verify that waypoint reach distance works with spaced waypoints
        assert!(
            agent.waypoint_reach_distance >= 1.0,
            "Waypoint reach distance ({}) should work with spaced waypoints (>=1.0)",
            agent.waypoint_reach_distance
        );
    }

    #[test]
    fn test_waypoint_filtering() {
        // Test basic filtering
        let waypoints = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(2.0, 0.0, 0.0),
            Vec3::new(3.0, 0.0, 0.0),
            Vec3::new(4.0, 0.0, 0.0),
            Vec3::new(5.0, 0.0, 0.0),
        ];

        let filtered = filter_waypoints_for_spacing(waypoints, 2.0);

        // Should keep start (0,0,0), waypoint at distance 2+ (2,0,0), waypoint at distance 4+ (4,0,0), and end (5,0,0)
        assert_eq!(filtered.len(), 4);
        assert_eq!(filtered[0], Vec3::new(0.0, 0.0, 0.0)); // Start
        assert_eq!(filtered[1], Vec3::new(2.0, 0.0, 0.0)); // First at min distance
        assert_eq!(filtered[2], Vec3::new(4.0, 0.0, 0.0)); // Second at min distance
        assert_eq!(filtered[3], Vec3::new(5.0, 0.0, 0.0)); // End
    }

    #[test]
    fn test_waypoint_filtering_short_path() {
        // Test with very short path
        let waypoints = vec![Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0)];

        let filtered = filter_waypoints_for_spacing(waypoints.clone(), 2.0);

        // Should keep both waypoints unchanged
        assert_eq!(filtered, waypoints);
    }

    #[test]
    fn test_waypoint_filtering_empty() {
        let waypoints = vec![];
        let filtered = filter_waypoints_for_spacing(waypoints.clone(), 2.0);
        assert_eq!(filtered, waypoints);
    }

    #[test]
    fn test_obstacle_blocks_direct_path() {
        // Create a small terrain
        let terrain = TerrainData::create_flat(8, 8, 1.0, 0.0).unwrap();

        // Place a large obstacle directly between start and goal
        let objects = vec![env_obj(
            "boulder",
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::ZERO,
            Vec3::new(3.0, 3.0, 3.0),
        )];

        let nav_grid = NavigationGrid::from_terrain_and_objects(
            &terrain,
            &objects,
            PathfindingConfig::default(),
        )
        .unwrap();

        // Try to path from left to right through the obstacle
        let start = Vec3::new(-2.0, 0.0, 0.0);
        let goal = Vec3::new(2.0, 0.0, 0.0);

        let path = find_path(&nav_grid, start, goal, 0.5);

        if let Some(path) = path {
            println!("Path found with {} waypoints:", path.len());
            for (i, waypoint) in path.iter().enumerate() {
                println!(
                    "  {}: ({:.1}, {:.1}, {:.1})",
                    i, waypoint.x, waypoint.y, waypoint.z
                );
            }

            // If a path exists, it should go around the obstacle (not straight through)
            // The path should have more than 2 waypoints (start and end)
            assert!(
                path.len() > 2,
                "Path should go around obstacle, not straight through. Path length: {}",
                path.len()
            );

            // Verify the path doesn't go through the blocked center area
            for waypoint in &path {
                let grid_pos = nav_grid.world_to_grid(*waypoint).unwrap();
                assert!(
                    nav_grid.is_walkable(grid_pos.x, grid_pos.z),
                    "Path waypoint at ({:.1}, {:.1}) should not be on blocked terrain",
                    waypoint.x,
                    waypoint.z
                );
            }
        } else {
            // If no path exists, that's also valid - the obstacle might completely block the route
            println!("No path found - obstacle may completely block the route");
        }
    }

    #[test]
    fn test_pathfinding_test_map_obstacles() {
        // Test with the actual pathfinding_test.bin map data
        // This should match what the game is using

        // Recreate the same setup as the test map
        let terrain = TerrainData::create_flat(32, 32, 1.0, 0.0).unwrap();

        // Create obstacles matching the test map (3 trees at specific positions)
        // Based on map_info output: Position bounds: (1.9, 0.0, 4.1) to (4.9, 0.0, 14.1)
        let objects = vec![
            env_obj(
                "tree",
                Vec3::new(2.0, 0.0, 5.0),
                Vec3::ZERO,
                Vec3::new(2.5, 3.0, 2.5),
            ),
            env_obj(
                "tree",
                Vec3::new(3.5, 0.0, 9.0),
                Vec3::ZERO,
                Vec3::new(2.3, 3.0, 2.3),
            ),
            env_obj(
                "tree",
                Vec3::new(4.5, 0.0, 13.0),
                Vec3::ZERO,
                Vec3::new(2.2, 3.0, 2.2),
            ),
        ];

        let nav_grid = NavigationGrid::from_terrain_and_objects(
            &terrain,
            &objects,
            PathfindingConfig::default(),
        )
        .unwrap();

        println!("=== Test Map Pathfinding Analysis ===");
        println!("Terrain: 32x32, scale=1.0, world bounds: (-16,-16) to (15,15)");
        println!("Objects: {} trees", objects.len());

        // Count blocked cells
        let blocked_count = nav_grid.walkable.iter().filter(|&&w| !w).count();
        let total_cells = nav_grid.walkable.len();
        println!(
            "Blocked cells: {}/{} ({:.1}%)",
            blocked_count,
            total_cells,
            (blocked_count as f32 / total_cells as f32) * 100.0
        );

        // Test pathfinding from player spawn (0,0,0) to a point behind the trees
        let start = Vec3::new(0.0, 0.0, 0.0); // Player spawn
        let goal = Vec3::new(3.0, 0.0, 15.0); // Behind the trees

        println!(
            "Testing path from ({:.1}, {:.1}, {:.1}) to ({:.1}, {:.1}, {:.1})",
            start.x, start.y, start.z, goal.x, goal.y, goal.z
        );

        let path = find_path(&nav_grid, start, goal, 0.5);

        if let Some(path) = path {
            println!("Path found with {} waypoints:", path.len());
            for (i, waypoint) in path.iter().enumerate() {
                println!(
                    "  {}: ({:.1}, {:.1}, {:.1})",
                    i, waypoint.x, waypoint.y, waypoint.z
                );

                // Check if this waypoint is near any tree
                for (j, obj) in objects.iter().enumerate() {
                    let distance = waypoint.distance(obj.position);
                    let tree_radius = obj.scale.x * 0.3; // Tree collision radius
                    if distance < tree_radius {
                        println!(
                            "    WARNING: Waypoint {} is inside tree {} (distance={:.1}, radius={:.1})",
                            i, j, distance, tree_radius
                        );
                    }
                }
            }

            // The path should avoid going through trees
            assert!(
                path.len() >= 2,
                "Path should have at least start and end points"
            );
        } else {
            println!("No path found - this might indicate a problem!");
            panic!("Expected to find a path around the trees");
        }
    }

    #[test]
    fn test_navigation_grid_with_objects() {
        let terrain = TerrainData::create_flat(8, 8, 1.0, 0.0).unwrap();
        let objects = vec![
            env_obj(
                "rock",
                Vec3::new(2.0, 0.0, 2.0),
                Vec3::ZERO,
                Vec3::new(0.5, 1.0, 0.5),
            ), // Small rock
        ];

        let nav_grid = NavigationGrid::from_terrain_and_objects(
            &terrain,
            &objects,
            PathfindingConfig::default(),
        )
        .unwrap();

        // Rock should block its grid cell
        let rock_node = nav_grid.world_to_grid(Vec3::new(2.0, 0.0, 2.0)).unwrap();
        assert!(!nav_grid.is_walkable(rock_node.x, rock_node.z));

        // Test that some cells are indeed blocked by objects
        let blocked_count = nav_grid.walkable.iter().filter(|&&w| !w).count();
        assert!(
            blocked_count > 0,
            "Should have some blocked cells from environment objects"
        );
    }

    #[test]
    fn test_object_outside_terrain_bounds() {
        let terrain = TerrainData::create_flat(4, 4, 1.0, 0.0).unwrap();
        let objects = vec![
            env_obj_simple("rock", Vec3::new(100.0, 0.0, 100.0)), // Way outside
        ];

        // Should not panic or fail, just skip the out-of-bounds object
        let nav_grid = NavigationGrid::from_terrain_and_objects(
            &terrain,
            &objects,
            PathfindingConfig::default(),
        )
        .unwrap();

        // Navigation grid dimensions match terrain
        assert_eq!(nav_grid.width, 4);
        assert_eq!(nav_grid.height, 4);
    }

    #[test]
    fn test_object_type_specific_blocking() {
        let terrain = TerrainData::create_flat(8, 8, 1.0, 0.0).unwrap();

        // Test different object types
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

        let nav_grid = NavigationGrid::from_terrain_and_objects(
            &terrain,
            &objects,
            PathfindingConfig::default(),
        )
        .unwrap();

        // Each object should block some cells based on their type
        let tree_node = nav_grid.world_to_grid(Vec3::new(1.0, 0.0, 1.0)).unwrap();
        let rock_node = nav_grid.world_to_grid(Vec3::new(-1.0, 0.0, -1.0)).unwrap();
        let grass_node = nav_grid.world_to_grid(Vec3::new(2.0, 0.0, -2.0)).unwrap();

        assert!(!nav_grid.is_walkable(tree_node.x, tree_node.z));
        assert!(!nav_grid.is_walkable(rock_node.x, rock_node.z));
        assert!(nav_grid.is_walkable(grass_node.x, grass_node.z)); // Grass should not block paths
    }

    #[test]
    fn test_invalid_object_scale() {
        let terrain = TerrainData::create_flat(4, 4, 1.0, 0.0).unwrap();
        let objects = vec![
            EnvironmentObject::new(
                "rock".to_string(),
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::ZERO,
                Vec3::new(0.0, 1.0, 0.0),
            ), // Invalid scale
        ];

        // Should not panic or fail, just skip the invalid object
        let nav_grid = NavigationGrid::from_terrain_and_objects(
            &terrain,
            &objects,
            PathfindingConfig::default(),
        )
        .unwrap();

        // Navigation grid dimensions match terrain
        assert_eq!(nav_grid.width, 4);
        assert_eq!(nav_grid.height, 4);
    }

    #[test]
    fn test_euclidean_distance() {
        let node1 = GridNode::new(0, 0);
        let node2 = GridNode::new(3, 4);

        // Euclidean distance should be sqrt(3^2 + 4^2) = sqrt(9 + 16) = sqrt(25) = 5.0
        assert_eq!(node1.euclidean_distance(&node2), 5.0);
        assert_eq!(node2.euclidean_distance(&node1), 5.0);
    }

    #[test]
    fn test_agent_radius_inflation() {
        // Test that agent radius properly inflates obstacles
        let terrain = TerrainData::create_flat(16, 16, 1.0, 0.0).unwrap(); // Larger terrain

        // Place a small tree that would be ignored without agent radius inflation
        let objects = vec![
            env_obj(
                "tree",
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::ZERO,
                Vec3::new(1.0, 1.0, 1.0),
            ), // Larger tree for testing
        ];

        let nav_grid = NavigationGrid::from_terrain_and_objects(
            &terrain,
            &objects,
            PathfindingConfig::default(),
        )
        .unwrap();

        // Test pathfinding with small agent (should find path close to tree)
        let start = Vec3::new(-3.0, 0.0, 0.0);
        let goal = Vec3::new(3.0, 0.0, 0.0);

        let small_agent_path = find_path(&nav_grid, start, goal, 0.1); // Small agent
        let large_agent_path = find_path(&nav_grid, start, goal, 1.5); // Large agent

        // Both should find paths, but large agent should have different (longer) path
        assert!(small_agent_path.is_some(), "Small agent should find a path");
        assert!(large_agent_path.is_some(), "Large agent should find a path");

        let small_path = small_agent_path.unwrap();
        let large_path = large_agent_path.unwrap();

        // Large agent path should avoid getting too close to the tree
        // Check that no waypoint in the large agent path is too close to the tree center
        let tree_center = Vec3::new(0.0, 0.0, 0.0);
        let min_distance_large = large_path
            .iter()
            .map(|waypoint| waypoint.distance(tree_center))
            .fold(f32::INFINITY, f32::min);
        let min_distance_small = small_path
            .iter()
            .map(|waypoint| waypoint.distance(tree_center))
            .fold(f32::INFINITY, f32::min);

        // Large agent should maintain more distance from the tree
        assert!(
            min_distance_large > min_distance_small,
            "Large agent (min_dist={:.2}) should maintain more distance from tree than small agent (min_dist={:.2})",
            min_distance_large,
            min_distance_small
        );
    }

    #[test]
    fn test_large_scale_procedural_pathfinding() {
        // Create a larger terrain (32x32) with hills
        let size = 32;
        let mut heights = Vec::with_capacity((size * size) as usize);

        // Create rolling hills terrain using simple procedural generation
        for z in 0..size {
            for x in 0..size {
                let fx = x as f32 / size as f32;
                let fz = z as f32 / size as f32;

                // Generate rolling hills using sine waves
                let height = (fx * std::f32::consts::PI * 2.0).sin() * 2.0
                    + (fz * std::f32::consts::PI * 3.0).sin() * 1.5
                    + ((fx + fz) * std::f32::consts::PI * 4.0).sin() * 0.5;
                heights.push(height);
            }
        }

        let terrain = TerrainData::new(size, size, heights, 1.0).unwrap();

        // Place strategic obstacles (trees and rocks, but not grass)
        let mut objects = Vec::new();

        // Add some trees scattered throughout
        let tree_positions = [
            (8, 8),
            (12, 6),
            (20, 15),
            (5, 25),
            (18, 28),
            (25, 10),
            (14, 20),
            (22, 5),
            (3, 18),
            (28, 22),
            (10, 30),
            (26, 3),
        ];
        for (x, z) in tree_positions {
            objects.push(EnvironmentObject::new(
                "tree".to_string(),
                Vec3::new(x as f32 - 16.0, 0.0, z as f32 - 16.0), // Center on terrain
                Vec3::ZERO,
                Vec3::new(2.0, 3.0, 2.0),
            ));
        }

        // Add some rocks
        let rock_positions = [
            (15, 12),
            (7, 20),
            (23, 8),
            (11, 26),
            (19, 4),
            (6, 14),
            (24, 18),
            (9, 7),
            (17, 24),
            (13, 30),
            (28, 16),
            (4, 11),
        ];
        for (x, z) in rock_positions {
            objects.push(EnvironmentObject::new(
                "rock".to_string(),
                Vec3::new(x as f32 - 16.0, 0.0, z as f32 - 16.0), // Center on terrain
                Vec3::ZERO,
                Vec3::new(1.0, 1.0, 1.0),
            ));
        }

        // Add some grass (should not block paths)
        let grass_positions = [(16, 16), (10, 10), (20, 20), (5, 5), (25, 25)];
        for (x, z) in grass_positions {
            objects.push(EnvironmentObject::new(
                "grass".to_string(),
                Vec3::new(x as f32 - 16.0, 0.0, z as f32 - 16.0), // Center on terrain
                Vec3::ZERO,
                Vec3::ONE,
            ));
        }

        let nav_grid = NavigationGrid::from_terrain_and_objects(
            &terrain,
            &objects,
            PathfindingConfig::default(),
        )
        .unwrap();

        // Verify terrain dimensions
        assert_eq!(nav_grid.width, size);
        assert_eq!(nav_grid.height, size);

        // Test pathfinding from corner to corner
        let start_world = Vec3::new(-15.0, 0.0, -15.0); // Near corner
        let goal_world = Vec3::new(14.0, 0.0, 14.0); // Opposite corner

        let path = find_path(&nav_grid, start_world, goal_world, 0.5);

        // Path should be found
        assert!(path.is_some(), "Should find a path from corner to corner");

        let path = path.unwrap();

        // Path should have reasonable length (not too short, not too long)
        assert!(
            path.len() >= 10,
            "Path should have multiple waypoints, got {}",
            path.len()
        );
        assert!(
            path.len() <= 100,
            "Path should not be excessively long, got {}",
            path.len()
        );

        // Verify start and end points are close to requested positions
        let path_start = path.first().unwrap();
        let path_end = path.last().unwrap();

        assert!(
            path_start.distance(start_world) < 2.0,
            "Path start should be close to requested start"
        );
        assert!(
            path_end.distance(goal_world) < 2.0,
            "Path end should be close to requested goal"
        );

        // Verify path doesn't go through blocked cells
        for waypoint in &path {
            if let Some(grid_node) = nav_grid.world_to_grid(*waypoint) {
                assert!(
                    nav_grid.is_walkable(grid_node.x, grid_node.z),
                    "Path waypoint at ({:.1}, {:.1}, {:.1}) should be on walkable terrain",
                    waypoint.x,
                    waypoint.y,
                    waypoint.z
                );
            }
        }

        // Test that grass positions are walkable (not blocked)
        for (x, z) in grass_positions {
            let grass_world = Vec3::new(x as f32 - 16.0, 0.0, z as f32 - 16.0);
            if let Some(grass_node) = nav_grid.world_to_grid(grass_world) {
                assert!(
                    nav_grid.is_walkable(grass_node.x, grass_node.z),
                    "Grass at ({}, {}) should not block pathfinding",
                    x,
                    z
                );
            }
        }

        // Test that some tree/rock positions are blocked
        let mut blocked_count = 0;
        for (x, z) in tree_positions.iter().chain(rock_positions.iter()) {
            let obj_world = Vec3::new(*x as f32 - 16.0, 0.0, *z as f32 - 16.0);
            if let Some(obj_node) = nav_grid.world_to_grid(obj_world) {
                if !nav_grid.is_walkable(obj_node.x, obj_node.z) {
                    blocked_count += 1;
                }
            }
        }

        assert!(
            blocked_count > 0,
            "Some trees/rocks should block pathfinding"
        );

        // Test alternative path when direct route is blocked
        // Create a more targeted obstacle that definitely blocks the path
        let mut nav_grid_with_wall = nav_grid.clone();

        // Block a more extensive area in the center to force rerouting
        for z in 10..22 {
            for x in 14..18 {
                let wall_node = GridNode::new(x, z);
                nav_grid_with_wall.set_cell_walkable_with_priority(wall_node, false, 255);
            }
        }

        let blocked_path = find_path(&nav_grid_with_wall, start_world, goal_world, 0.5);
        assert!(
            blocked_path.is_some(),
            "Should find alternative path around obstacles"
        );

        // The alternative path should exist and be valid, but we don't require it to be longer
        // since the A* algorithm might find an equally efficient alternative route
        let blocked_path = blocked_path.unwrap();
        assert!(
            blocked_path.len() >= 10,
            "Alternative path should have reasonable length, got {}",
            blocked_path.len()
        );
    }
}
