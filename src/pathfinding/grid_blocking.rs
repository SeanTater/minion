//! Optimized grid blocking operations with priority support

use crate::pathfinding::{GridNode, NavigationGrid};
use bevy::prelude::*;

/// Block circular area with priority-based override system
pub fn block_circular_area_with_priority(
    nav_grid: &mut NavigationGrid,
    center: Vec3,
    radius: f32,
    priority: u8,
) {
    let Some(center_cell) = nav_grid.world_to_grid(center) else {
        return;
    };
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
    priority: u8,
) {
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
        let mut nav_grid =
            NavigationGrid::from_terrain(&terrain, PathfindingConfig::default()).unwrap();

        // Block with priority
        block_circular_area_with_priority(&mut nav_grid, Vec3::ZERO, 1.5, 100);
        let center_node = nav_grid.world_to_grid(Vec3::ZERO).unwrap();
        assert!(!nav_grid.is_walkable(center_node.x, center_node.z));

        // Check that priority is set correctly
        assert_eq!(nav_grid.get_obstacle_priority(center_node), 100);
    }

    #[test]
    fn test_rectangular_blocking_with_priority() {
        let terrain = TerrainData::create_flat(8, 8, 1.0, 0.0).unwrap();
        let mut nav_grid =
            NavigationGrid::from_terrain(&terrain, PathfindingConfig::default()).unwrap();

        // Block rectangular area
        block_rectangular_area_with_priority(
            &mut nav_grid,
            Vec3::ZERO,
            Vec3::new(1.0, 0.0, 1.0),
            150,
        );

        let center_node = nav_grid.world_to_grid(Vec3::ZERO).unwrap();
        assert!(!nav_grid.is_walkable(center_node.x, center_node.z));
        assert_eq!(nav_grid.get_obstacle_priority(center_node), 150);
    }

    #[test]
    fn test_priority_override_system() {
        let terrain = TerrainData::create_flat(8, 8, 1.0, 0.0).unwrap();
        let mut nav_grid =
            NavigationGrid::from_terrain(&terrain, PathfindingConfig::default()).unwrap();

        let center_node = nav_grid.world_to_grid(Vec3::ZERO).unwrap();

        // Block with lower priority first
        block_circular_area_with_priority(&mut nav_grid, Vec3::ZERO, 1.0, 100);
        assert!(!nav_grid.is_walkable(center_node.x, center_node.z));
        assert_eq!(nav_grid.get_obstacle_priority(center_node), 100);

        // Try to override with lower priority - should NOT work
        block_circular_area_with_priority(&mut nav_grid, Vec3::ZERO, 0.5, 50);
        assert_eq!(nav_grid.get_obstacle_priority(center_node), 100); // Should remain 100

        // Override with higher priority - should work
        block_circular_area_with_priority(&mut nav_grid, Vec3::ZERO, 0.5, 200);
        assert_eq!(nav_grid.get_obstacle_priority(center_node), 200); // Should be updated
    }

    #[test]
    fn test_out_of_bounds_blocking() {
        let terrain = TerrainData::create_flat(4, 4, 1.0, 0.0).unwrap();
        let mut nav_grid =
            NavigationGrid::from_terrain(&terrain, PathfindingConfig::default()).unwrap();

        // Try to block way outside terrain bounds - should not panic
        block_circular_area_with_priority(&mut nav_grid, Vec3::new(100.0, 0.0, 100.0), 5.0, 100);
        block_rectangular_area_with_priority(
            &mut nav_grid,
            Vec3::new(-100.0, 0.0, -100.0),
            Vec3::new(2.0, 0.0, 2.0),
            100,
        );

        // Grid should still be valid
        assert_eq!(nav_grid.width, 4);
        assert_eq!(nav_grid.height, 4);
    }
}
