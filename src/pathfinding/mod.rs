use crate::components::PathfindingAgent;
use crate::game_logic::errors::MinionResult;
use crate::map::{EnvironmentObject, TerrainData};
use crate::terrain::coordinates::*;
use bevy::prelude::*;
use pathfinding::prelude::astar;


/// Configuration for pathfinding grid generation
#[derive(Debug, Clone)]
pub struct PathfindingConfig {
    /// Maximum slope angle in degrees that is considered walkable
    pub max_walkable_slope: f32,
    /// Linear slope cost factor - higher values make slope more expensive
    pub slope_cost_factor: f32,
}

impl Default for PathfindingConfig {
    fn default() -> Self {
        Self {
            max_walkable_slope: 45.0, // 45 degrees max slope
            slope_cost_factor: 0.5,   // Linear slope cost factor
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
            width: terrain.width,
            height: terrain.height,
            cell_size: terrain.scale,
            terrain_width: terrain.width,
            terrain_height: terrain.height,
            terrain_scale: terrain.scale,
            config,
        };

        // Apply environment object blocking
        for obj in objects {
            Self::project_object_to_grid(obj, &mut nav_grid);
        }

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
    fn world_to_grid(&self, world_pos: Vec3) -> Option<GridNode> {
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

    /// Set walkability for a specific grid cell
    fn set_cell_walkable(&mut self, node: GridNode, walkable: bool) {
        if node.x >= self.width || node.z >= self.height {
            return; // Out of bounds, ignore
        }
        let index = (node.z * self.width + node.x) as usize;
        if let Some(cell) = self.walkable.get_mut(index) {
            *cell = walkable;
        }
    }

    /// Project an environment object onto the navigation grid, blocking appropriate cells
    fn project_object_to_grid(obj: &EnvironmentObject, nav_grid: &mut NavigationGrid) {
        // Handle zero/negative scales gracefully
        if obj.scale.x <= 0.0 || obj.scale.z <= 0.0 {
            warn!(
                "Invalid object scale {:?} for {}, skipping",
                obj.scale, obj.object_type
            );
            return;
        }

        // Apply object-specific projection based on collider shape
        match obj.object_type.as_str() {
            "tree" => Self::block_circular_area(nav_grid, obj.position, obj.scale.x * 0.3),
            "rock" | "boulder" => {
                Self::block_circular_area(nav_grid, obj.position, obj.scale.x * 0.5)
            }
            "grass" => {
                // Grass is decorative - don't block pathfinding
                return;
            }
            _ => Self::block_rectangular_area(nav_grid, obj.position, obj.scale * 0.5),
        }
    }

    /// Block a circular area around a center point
    fn block_circular_area(nav_grid: &mut NavigationGrid, center: Vec3, radius: f32) {
        let Some(center_cell) = nav_grid.world_to_grid(center) else {
            return;
        };
        let cell_radius = (radius / nav_grid.cell_size).ceil() as i32;

        for dz in -cell_radius..=cell_radius {
            for dx in -cell_radius..=cell_radius {
                let x = center_cell.x as i32 + dx;
                let z = center_cell.z as i32 + dz;

                if x >= 0 && z >= 0 && x < nav_grid.width as i32 && z < nav_grid.height as i32 {
                    let cell = GridNode::new(x as u32, z as u32);

                    // Convert cell back to world for distance check
                    let half_width = (nav_grid.terrain_width as f32 * nav_grid.terrain_scale) / 2.0;
                    let half_height =
                        (nav_grid.terrain_height as f32 * nav_grid.terrain_scale) / 2.0;
                    let world_x = (x as f32 * nav_grid.terrain_scale) - half_width;
                    let world_z = (z as f32 * nav_grid.terrain_scale) - half_height;
                    let height = nav_grid.get_height_at_grid(cell).unwrap_or(0.0);

                    if center.distance(Vec3::new(world_x, height, world_z)) <= radius {
                        nav_grid.set_cell_walkable(cell, false);
                    }
                }
            }
        }
    }

    /// Block a rectangular area around a center point
    fn block_rectangular_area(nav_grid: &mut NavigationGrid, center: Vec3, half_extents: Vec3) {
        let min_world = center - half_extents;
        let max_world = center + half_extents;

        let Some(min_cell) = nav_grid.world_to_grid(min_world) else {
            return;
        };
        let Some(max_cell) = nav_grid.world_to_grid(max_world) else {
            return;
        };

        for z in min_cell.z..=max_cell.z {
            for x in min_cell.x..=max_cell.x {
                nav_grid.set_cell_walkable(GridNode::new(x, z), false);
            }
        }
    }
}

/// Find a path between two world positions using A* pathfinding
pub fn find_path(
    navigation_grid: &NavigationGrid,
    start_world: Vec3,
    goal_world: Vec3,
) -> Option<Vec<Vec3>> {
    let start_node = navigation_grid.world_to_grid(start_world)?;
    let goal_node = navigation_grid.world_to_grid(goal_world)?;

    // Check if start and goal are walkable
    if !navigation_grid.is_walkable(start_node.x, start_node.z)
        || !navigation_grid.is_walkable(goal_node.x, goal_node.z)
    {
        return None;
    }

    // Use A* to find the path
    let (path, _cost) = astar(
        &start_node,
        |node| {
            let current_node = *node;
            node.neighbors(navigation_grid.width, navigation_grid.height)
                .into_iter()
                .filter(|neighbor| navigation_grid.is_walkable(neighbor.x, neighbor.z))
                .map(move |neighbor| {
                    (
                        neighbor,
                        navigation_grid.movement_cost(current_node, neighbor),
                    )
                })
        },
        |node| (node.euclidean_distance(&goal_node) * 10.0) as u32,
        |node| *node == goal_node,
    )?;

    // Convert grid path to world coordinates
    let half_width = (navigation_grid.terrain_width as f32 * navigation_grid.terrain_scale) / 2.0;
    let half_height = (navigation_grid.terrain_height as f32 * navigation_grid.terrain_scale) / 2.0;

    Some(
        path.into_iter()
            .map(|node| {
                let world_x = (node.x as f32 * navigation_grid.terrain_scale) - half_width;
                let world_z = (node.z as f32 * navigation_grid.terrain_scale) - half_height;
                let height = navigation_grid.get_height_at_grid(node).unwrap_or(0.0);
                Vec3::new(world_x, height, world_z)
            })
            .collect(),
    )
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
        if let Some(last_waypoint) = agent.current_path.last() {
            if last_waypoint.distance(destination) > agent.max_path_distance {
                return true;
            }
        } else if !agent.current_path.is_empty() {
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
            let distance = transform.translation.distance(current_waypoint);

            // Debug logging commented out to reduce console spam
            // debug!("Pathfinding: current_waypoint_index={}/{}, distance_to_waypoint={:.2}",
            //        agent.path_index, agent.current_path.len(), distance);

            if distance <= agent.waypoint_reach_distance {
                let old_index = agent.path_index;
                agent.advance_waypoint();
                info!(
                    "Waypoint reached! Advanced from index {} to {} (path length: {})",
                    old_index,
                    agent.path_index,
                    agent.current_path.len()
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
            // if !agent.current_path.is_empty() {
            //     debug!("No current waypoint available (index={}, len={})",
            //            agent.path_index, agent.current_path.len());
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
                if let Some(new_path) =
                    find_path(&navigation_grid, transform.translation, destination)
                {
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

        assert!(agent.current_path.is_empty());
        assert_eq!(agent.path_index, 0);
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
        // Verify that waypoint reach distance is smaller than typical stopping distance
        assert!(
            agent.waypoint_reach_distance < 0.5,
            "Waypoint reach distance ({}) should be smaller than player stopping distance (0.5)",
            agent.waypoint_reach_distance
        );
    }

    #[test]
    fn test_navigation_grid_with_objects() {
        let terrain = TerrainData::create_flat(8, 8, 1.0, 0.0).unwrap();
        let objects = vec![
            EnvironmentObject::new(
                "rock".to_string(),
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
    fn test_circular_blocking_area() {
        let terrain = TerrainData::create_flat(8, 8, 1.0, 0.0).unwrap();
        let config = PathfindingConfig::default();
        let mut nav_grid = NavigationGrid::from_terrain(&terrain, config).unwrap();

        // Test circular blocking
        NavigationGrid::block_circular_area(&mut nav_grid, Vec3::new(0.0, 0.0, 0.0), 1.0);

        // Center should be blocked
        let center_node = nav_grid.world_to_grid(Vec3::new(0.0, 0.0, 0.0)).unwrap();
        assert!(!nav_grid.is_walkable(center_node.x, center_node.z));
    }

    #[test]
    fn test_rectangular_blocking_area() {
        let terrain = TerrainData::create_flat(8, 8, 1.0, 0.0).unwrap();
        let config = PathfindingConfig::default();
        let mut nav_grid = NavigationGrid::from_terrain(&terrain, config).unwrap();

        // Test rectangular blocking
        NavigationGrid::block_rectangular_area(
            &mut nav_grid,
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 1.0),
        );

        // Center should be blocked
        let center_node = nav_grid.world_to_grid(Vec3::new(0.0, 0.0, 0.0)).unwrap();
        assert!(!nav_grid.is_walkable(center_node.x, center_node.z));
    }

    #[test]
    fn test_object_outside_terrain_bounds() {
        let terrain = TerrainData::create_flat(4, 4, 1.0, 0.0).unwrap();
        let objects = vec![
            EnvironmentObject::simple("rock".to_string(), Vec3::new(100.0, 0.0, 100.0)), // Way outside
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

        let path = find_path(&nav_grid, start_world, goal_world);

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
                nav_grid_with_wall.set_cell_walkable(wall_node, false);
            }
        }

        let blocked_path = find_path(&nav_grid_with_wall, start_world, goal_world);
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
