use crate::map::TerrainData;

/// Grid coordinates (unsigned integers)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridCoord {
    pub x: u32,
    pub z: u32,
}

/// World coordinates (floating point)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldCoord {
    pub x: f32,
    pub z: f32,
}

impl GridCoord {
    pub fn new(x: u32, z: u32) -> Self {
        Self { x, z }
    }

    /// Check if these coordinates are valid for the given terrain
    pub fn is_valid_for(&self, terrain: &TerrainData) -> bool {
        self.x < terrain.width && self.z < terrain.height
    }

    /// Convert to world coordinates
    pub fn to_world(&self, terrain: &TerrainData) -> WorldCoord {
        grid_to_world(terrain, self.x as f32, self.z as f32)
    }
}

impl WorldCoord {
    pub fn new(x: f32, z: f32) -> Self {
        Self { x, z }
    }

    /// Convert to grid coordinates
    pub fn to_grid(&self, terrain: &TerrainData) -> Option<GridCoord> {
        let (grid_x, grid_z) = world_to_grid(terrain, self.x, self.z);
        if grid_x >= 0.0 && grid_z >= 0.0 && grid_x < terrain.width as f32 && grid_z < terrain.height as f32 {
            Some(GridCoord::new(grid_x as u32, grid_z as u32))
        } else {
            None
        }
    }
}

/// Convert world coordinates to grid coordinates, accounting for terrain centering
pub fn world_to_grid(terrain: &TerrainData, world_x: f32, world_z: f32) -> (f32, f32) {
    let terrain_width = terrain.width as f32 * terrain.scale;
    let terrain_height = terrain.height as f32 * terrain.scale;
    let center_x_offset = terrain_width / 2.0;
    let center_z_offset = terrain_height / 2.0;

    let grid_x = (world_x + center_x_offset) / terrain.scale;
    let grid_z = (world_z + center_z_offset) / terrain.scale;
    (grid_x, grid_z)
}

/// Convert world coordinates to grid coordinates, returning typed coordinates
pub fn world_to_grid_coord(terrain: &TerrainData, world: WorldCoord) -> Option<GridCoord> {
    world.to_grid(terrain)
}

/// Convert grid coordinates to world coordinates, accounting for terrain centering
pub fn grid_to_world(terrain: &TerrainData, grid_x: f32, grid_z: f32) -> WorldCoord {
    let terrain_width = terrain.width as f32 * terrain.scale;
    let terrain_height = terrain.height as f32 * terrain.scale;
    let center_x_offset = terrain_width / 2.0;
    let center_z_offset = terrain_height / 2.0;

    let world_x = grid_x * terrain.scale - center_x_offset;
    let world_z = grid_z * terrain.scale - center_z_offset;
    WorldCoord::new(world_x, world_z)
}

/// Check if grid coordinates are within terrain bounds
pub fn is_valid_grid(terrain: &TerrainData, grid_x: f32, grid_z: f32) -> bool {
    grid_x >= 0.0 && grid_z >= 0.0 
        && grid_x < terrain.width as f32 
        && grid_z < terrain.height as f32
}

/// Get height at exact grid position (no interpolation)
pub fn get_height_at_grid(terrain: &TerrainData, x: u32, z: u32) -> Option<f32> {
    if x >= terrain.width || z >= terrain.height {
        return None;
    }
    let index = (z * terrain.width + x) as usize;
    terrain.heights.get(index).copied()
}

/// Get height at grid coordinate with type safety
pub fn get_height_at_grid_coord(terrain: &TerrainData, coord: GridCoord) -> Option<f32> {
    get_height_at_grid(terrain, coord.x, coord.z)
}

/// Get interpolated height at world position using bilinear interpolation
pub fn get_height_at_world_interpolated(terrain: &TerrainData, world_x: f32, world_z: f32) -> Option<f32> {
    let (grid_x, grid_z) = world_to_grid(terrain, world_x, world_z);

    // Check bounds (need at least 1 grid cell margin for interpolation)
    if grid_x < 0.0 || grid_z < 0.0 
        || grid_x >= (terrain.width - 1) as f32 
        || grid_z >= (terrain.height - 1) as f32 {
        return None;
    }

    // Bilinear interpolation
    let x0 = grid_x.floor() as u32;
    let z0 = grid_z.floor() as u32;
    let x1 = x0 + 1;
    let z1 = z0 + 1;

    let fx = grid_x.fract();
    let fz = grid_z.fract();

    let h00 = get_height_at_grid(terrain, x0, z0)?;
    let h10 = get_height_at_grid(terrain, x1, z0)?;
    let h01 = get_height_at_grid(terrain, x0, z1)?;
    let h11 = get_height_at_grid(terrain, x1, z1)?;

    let h0 = h00 * (1.0 - fx) + h10 * fx;
    let h1 = h01 * (1.0 - fx) + h11 * fx;

    Some(h0 * (1.0 - fz) + h1 * fz)
}

/// Get interpolated height at world coordinate with type safety
pub fn get_height_at_world_coord(terrain: &TerrainData, world: WorldCoord) -> Option<f32> {
    get_height_at_world_interpolated(terrain, world.x, world.z)
}

/// Get height at world position using nearest neighbor (faster, less accurate)
pub fn get_height_at_world_nearest(terrain: &TerrainData, world_x: f32, world_z: f32) -> Option<f32> {
    let (grid_x, grid_z) = world_to_grid(terrain, world_x, world_z);

    if !is_valid_grid(terrain, grid_x, grid_z) {
        return None;
    }

    let x = grid_x.round() as u32;
    let z = grid_z.round() as u32;
    get_height_at_grid(terrain, x, z)
}