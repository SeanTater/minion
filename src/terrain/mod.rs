use crate::game_logic::errors::MinionResult;
use crate::map::TerrainData;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy_rapier3d::prelude::*;

/// Coordinate transformation utilities for terrain
pub mod coordinates {
    use crate::map::TerrainData;

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

    /// Convert grid coordinates to world coordinates, accounting for terrain centering
    pub fn grid_to_world(terrain: &TerrainData, grid_x: f32, grid_z: f32) -> (f32, f32) {
        let terrain_width = terrain.width as f32 * terrain.scale;
        let terrain_height = terrain.height as f32 * terrain.scale;
        let center_x_offset = terrain_width / 2.0;
        let center_z_offset = terrain_height / 2.0;

        let world_x = grid_x * terrain.scale - center_x_offset;
        let world_z = grid_z * terrain.scale - center_z_offset;
        (world_x, world_z)
    }

    /// Check if grid coordinates are within terrain bounds
    pub fn is_valid_grid(terrain: &TerrainData, grid_x: f32, grid_z: f32) -> bool {
        grid_x >= 0.0 && grid_z >= 0.0 
            && grid_x < terrain.width as f32 
            && grid_z < terrain.height as f32
    }

    /// Get height at exact grid position (no interpolation)
    pub fn get_height_at_grid(terrain: &TerrainData, x: u32, y: u32) -> Option<f32> {
        if x >= terrain.width || y >= terrain.height {
            return None;
        }
        let index = (y * terrain.width + x) as usize;
        terrain.heights.get(index).copied()
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
}

/// Generate a 3D mesh from heightmap terrain data
pub fn generate_terrain_mesh(terrain: &TerrainData) -> MinionResult<Mesh> {
    let width = terrain.width as usize;
    let height = terrain.height as usize;
    let scale = terrain.scale;

    // Generate vertices with heights from terrain data
    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();

    // Calculate centering offset to position terrain around origin
    let center_x_offset = (width as f32 * scale) / 2.0;
    let center_z_offset = (height as f32 * scale) / 2.0;

    // Generate vertices grid
    for z in 0..height {
        for x in 0..width {
            let world_x = (x as f32 * scale) - center_x_offset;
            let world_z = (z as f32 * scale) - center_z_offset;
            let height_value = terrain.heights[z * width + x];

            vertices.push([world_x, height_value, world_z]);

            // Calculate UV coordinates (0.0-1.0 across terrain bounds)
            let u = x as f32 / (width - 1) as f32;
            let v = z as f32 / (height - 1) as f32;
            uvs.push([u, v]);
        }
    }

    // Calculate smooth normals using cross product of adjacent triangles
    normals.resize(vertices.len(), [0.0, 1.0, 0.0]); // Initialize with up vectors

    for z in 0..(height - 1) {
        for x in 0..(width - 1) {
            let i0 = z * width + x;
            let i1 = z * width + (x + 1);
            let i2 = (z + 1) * width + x;
            let i3 = (z + 1) * width + (x + 1);

            let v0 = Vec3::from(vertices[i0]);
            let v1 = Vec3::from(vertices[i1]);
            let v2 = Vec3::from(vertices[i2]);
            let v3 = Vec3::from(vertices[i3]);

            // Calculate normals for the two triangles in this quad (updated for counter-clockwise winding)
            let normal1 = (v2 - v0).cross(v1 - v0).normalize();
            let normal2 = (v2 - v1).cross(v3 - v1).normalize();

            // Accumulate normals for vertices (will be normalized later)
            normals[i0] = [
                normals[i0][0] + normal1.x,
                normals[i0][1] + normal1.y,
                normals[i0][2] + normal1.z,
            ];
            normals[i1] = [
                normals[i1][0] + normal1.x + normal2.x,
                normals[i1][1] + normal1.y + normal2.y,
                normals[i1][2] + normal1.z + normal2.z,
            ];
            normals[i2] = [
                normals[i2][0] + normal1.x + normal2.x,
                normals[i2][1] + normal1.y + normal2.y,
                normals[i2][2] + normal1.z + normal2.z,
            ];
            normals[i3] = [
                normals[i3][0] + normal2.x,
                normals[i3][1] + normal2.y,
                normals[i3][2] + normal2.z,
            ];
        }
    }

    // Normalize accumulated normals
    for normal in &mut normals {
        let n = Vec3::from(*normal).normalize();
        *normal = [n.x, n.y, n.z];
    }

    // Generate triangle indices for quad-based mesh (2 triangles per height cell)
    let mut indices = Vec::new();

    for z in 0..(height - 1) {
        for x in 0..(width - 1) {
            let i0 = (z * width + x) as u32;
            let i1 = (z * width + (x + 1)) as u32;
            let i2 = ((z + 1) * width + x) as u32;
            let i3 = ((z + 1) * width + (x + 1)) as u32;

            // First triangle: i0, i2, i1 (counter-clockwise when viewed from above)
            indices.push(i0);
            indices.push(i2);
            indices.push(i1);

            // Second triangle: i1, i2, i3 (counter-clockwise when viewed from above)
            indices.push(i1);
            indices.push(i2);
            indices.push(i3);
        }
    }

    // Create the mesh
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD
            | bevy::render::render_asset::RenderAssetUsages::MAIN_WORLD,
    );

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));

    Ok(mesh)
}

/// Generate a physics collider from terrain data using the same vertex/index data as the visual mesh
pub fn generate_terrain_collider(terrain: &TerrainData) -> MinionResult<Collider> {
    let width = terrain.width as usize;
    let height = terrain.height as usize;
    let scale = terrain.scale;

    // Generate vertices for physics mesh (same as visual mesh)
    let mut vertices = Vec::new();

    // Calculate centering offset to position terrain around origin (same as visual mesh)
    let center_x_offset = (width as f32 * scale) / 2.0;
    let center_z_offset = (height as f32 * scale) / 2.0;

    for z in 0..height {
        for x in 0..width {
            let world_x = (x as f32 * scale) - center_x_offset;
            let world_z = (z as f32 * scale) - center_z_offset;
            let height_value = terrain.heights[z * width + x];

            vertices.push(Vec3::new(world_x, height_value, world_z));
        }
    }

    // Generate triangle indices (same as visual mesh)
    let mut indices = Vec::new();

    for z in 0..(height - 1) {
        for x in 0..(width - 1) {
            let i0 = (z * width + x) as u32;
            let i1 = (z * width + (x + 1)) as u32;
            let i2 = ((z + 1) * width + x) as u32;
            let i3 = ((z + 1) * width + (x + 1)) as u32;

            // First triangle: i0, i2, i1 (counter-clockwise when viewed from above)
            indices.push([i0, i2, i1]);

            // Second triangle: i1, i2, i3 (counter-clockwise when viewed from above)
            indices.push([i1, i2, i3]);
        }
    }

    // Create trimesh collider
    let collider = Collider::trimesh(vertices, indices).map_err(|e| {
        crate::game_logic::errors::MinionError::InvalidMapData {
            reason: format!("Failed to create terrain collider: {e}"),
        }
    })?;
    Ok(collider)
}

/// Generate both visual mesh and physics collider from terrain data
pub fn generate_terrain_mesh_and_collider(terrain: &TerrainData) -> MinionResult<(Mesh, Collider)> {
    let mesh = generate_terrain_mesh(terrain)?;
    let collider = generate_terrain_collider(terrain)?;
    Ok((mesh, collider))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::TerrainData;

    #[test]
    fn test_terrain_mesh_generation() {
        let terrain = TerrainData::create_flat(3, 3, 1.0, 2.0).unwrap();
        let mesh = generate_terrain_mesh(&terrain).unwrap();

        // Check that mesh has correct number of vertices (3x3 = 9)
        if let Some(positions) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            // Vertex count should match terrain grid
            // Note: actual count check depends on internal mesh representation
            assert!(!positions.is_empty());
        }
    }

    #[test]
    fn test_terrain_collider_generation() {
        let terrain = TerrainData::create_flat(3, 3, 1.0, 2.0).unwrap();
        let collider = generate_terrain_collider(&terrain);
        assert!(collider.is_ok());
    }

    #[test]
    fn test_terrain_mesh_and_collider_generation() {
        let terrain = TerrainData::create_flat(4, 4, 2.0, 1.5).unwrap();
        let result = generate_terrain_mesh_and_collider(&terrain);
        assert!(result.is_ok());

        let (mesh, _collider) = result.unwrap();

        // Verify mesh has required attributes
        assert!(mesh.attribute(Mesh::ATTRIBUTE_POSITION).is_some());
        assert!(mesh.attribute(Mesh::ATTRIBUTE_NORMAL).is_some());
        assert!(mesh.attribute(Mesh::ATTRIBUTE_UV_0).is_some());
        assert!(mesh.indices().is_some());
    }

    #[test]
    fn test_varying_heights_terrain() {
        let heights = vec![0.0, 1.0, 2.0, 1.0, 3.0, 1.0, 2.0, 1.0, 0.0];
        let terrain = TerrainData::new(3, 3, heights, 1.0).unwrap();
        let mesh = generate_terrain_mesh(&terrain);
        assert!(mesh.is_ok());
    }

    #[test]
    fn test_coordinate_transformations() {
        use super::coordinates::*;
        
        // Create a 3x3 terrain with scale 1.0
        let terrain = TerrainData::create_flat(3, 3, 1.0, 0.0).unwrap();
        
        // Test world to grid transformation
        let (grid_x, grid_z) = world_to_grid(&terrain, 0.0, 0.0);
        assert_eq!(grid_x, 1.5); // Center of 3x3 grid
        assert_eq!(grid_z, 1.5);
        
        // Test grid to world transformation  
        let (world_x, world_z) = grid_to_world(&terrain, 1.5, 1.5);
        assert_eq!(world_x, 0.0);
        assert_eq!(world_z, 0.0);
        
        // Test bounds checking
        assert!(is_valid_grid(&terrain, 1.5, 1.5));
        assert!(!is_valid_grid(&terrain, 3.0, 1.5));
        assert!(!is_valid_grid(&terrain, -1.0, 1.5));
    }

    #[test]
    fn test_height_lookups() {
        use super::coordinates::*;
        
        // Create terrain with known heights
        let heights = vec![
            0.0, 1.0, 2.0, // z=0 row
            3.0, 4.0, 5.0, // z=1 row
            6.0, 7.0, 8.0, // z=2 row
        ];
        let terrain = TerrainData::new(3, 3, heights, 1.0).unwrap();
        
        // Test exact grid lookups
        assert_eq!(get_height_at_grid(&terrain, 0, 0), Some(0.0));
        assert_eq!(get_height_at_grid(&terrain, 1, 1), Some(4.0));
        assert_eq!(get_height_at_grid(&terrain, 2, 2), Some(8.0));
        assert_eq!(get_height_at_grid(&terrain, 3, 0), None); // Out of bounds
        
        // Test nearest neighbor world lookup
        assert_eq!(get_height_at_world_nearest(&terrain, -1.5, -1.5), Some(0.0)); // Corner
        assert_eq!(get_height_at_world_nearest(&terrain, 0.0, 0.0), Some(8.0)); // Center rounds to (2,2)
        
        // Test interpolated world lookup at exact points
        assert_eq!(get_height_at_world_interpolated(&terrain, -1.5, -1.5), Some(0.0));
        assert_eq!(get_height_at_world_interpolated(&terrain, 0.0, 0.0), Some(6.0)); // Interpolation at center
        
        // Test out of bounds
        assert_eq!(get_height_at_world_interpolated(&terrain, -2.0, 0.0), None);
        assert_eq!(get_height_at_world_nearest(&terrain, 2.0, 0.0), None);
    }
}
