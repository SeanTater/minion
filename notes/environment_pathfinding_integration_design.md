# Environment Objects & Pathfinding Integration Design

## Executive Summary

**Problem**: Navigation grid only considers terrain height/slope for walkability, ignoring environment objects (rocks, trees) that have physics colliders. This causes pathfinding to route through solid objects, leading to player getting stuck.

**Solution**: Integrate environment object placement data into navigation grid generation using a deferred grid building approach with efficient object-to-grid projection.

**Key Decision**: Use **Option C** - Build navigation grid from map data before objects spawn, with optimized geometric projection.

## Current Architecture Analysis

### Environment Object System (`src/plugins/environment.rs`)
- **Data Source**: `MapDefinition.environment_objects` (Vec<EnvironmentObject>)
- **Object Types**: "rock", "tree", "boulder", "grass" with varying collider shapes
- **Collider Mapping**: 
  - Trees: `Cylinder(height: scale.y * 1.5, radius: scale.x * 0.3)`
  - Rocks/Boulders: `Ball(radius: scale.x * 0.5)`  
  - Grass: `Ball(radius: scale.x * 0.1)` (minimal blocking)
  - Default: `Cuboid(scale * 0.5)`
- **Spawn Timing**: After map loading via `spawn_environment_objects.after(load_map)`

### Navigation Grid System (`src/pathfinding/mod.rs`)
- **Grid Structure**: 64x64 fixed resolution (4096 cells)
- **Current Logic**: `NavigationGrid::from_terrain()` - terrain-only walkability
- **Build Timing**: During `load_map()` immediately after terrain creation
- **Cell Size**: `(terrain_width * terrain_scale) / 64` world units per cell

### Integration Points
- **Map Loading**: `load_map()` in `src/plugins/map_loader.rs` (lines 17-90)
- **Grid Creation**: `NavigationGrid::from_terrain()` (lines 26, 52, 74)
- **Object Data**: Available in `MapDefinition` before object spawning

## Architectural Design Decision

### **Selected Approach: Option C - Build Grid from Map Data**

**Rationale**: 
- Cleanest separation of concerns - pathfinding logic doesn't need to query Bevy entities
- Best performance - single pass grid building with all data available
- Most maintainable - no complex timing dependencies or entity queries
- Extensible - easy to add object-specific pathfinding behaviors

### **Alternative Approaches Rejected**:

**Option A (Query Spawned Objects)**: 
- ❌ Complex entity queries during grid building
- ❌ Timing dependency issues (objects not fully spawned)
- ❌ ECS coupling makes testing difficult

**Option B (Update Grid After Objects)**: 
- ❌ Two-pass grid building complexity
- ❌ Risk of pathfinding using incomplete grid
- ❌ More complex error handling

**Option D (Hybrid)**: 
- ❌ Unnecessary complexity for current requirements
- ❌ Multiple code paths to maintain

## Technical Implementation Design

### 1. Object Shape Representation Strategy

**Selected**: **Scale-Aware Geometric Projection**

Each object type projects into navigation grid cells based on its actual collider shape:

```rust
fn project_object_to_grid(
    obj: &EnvironmentObject,
    nav_grid: &mut NavigationGrid
) {
    let world_pos = obj.position;
    let scale = obj.scale;
    
    match obj.object_type.as_str() {
        "tree" => {
            // Cylinder: height=scale.y*1.5, radius=scale.x*0.3
            let radius = scale.x * 0.3;
            block_circular_area(nav_grid, world_pos, radius);
        }
        "rock" | "boulder" => {
            // Ball: radius=scale.x*0.5  
            let radius = scale.x * 0.5;
            block_circular_area(nav_grid, world_pos, radius);
        }
        "grass" => {
            // Minimal blocking - only block exact cell
            if let Some(cell) = nav_grid.world_to_grid(world_pos) {
                set_cell_walkable(nav_grid, cell, false);
            }
        }
        _ => {
            // Cuboid: scale*0.5
            let half_extents = scale * 0.5;
            block_rectangular_area(nav_grid, world_pos, half_extents);
        }
    }
}
```

### 2. Efficient Geometric Algorithms

**Circular Area Blocking** (for rocks, trees):
```rust
fn block_circular_area(nav_grid: &mut NavigationGrid, center: Vec3, radius: f32) {
    let center_cell = nav_grid.world_to_grid(center)?;
    let cell_radius = (radius / nav_grid.cell_size).ceil() as i32;
    
    for dz in -cell_radius..=cell_radius {
        for dx in -cell_radius..=cell_radius {
            let cell_x = center_cell.x as i32 + dx;
            let cell_z = center_cell.z as i32 + dz;
            
            if cell_x >= 0 && cell_z >= 0 && 
               cell_x < nav_grid.width as i32 && cell_z < nav_grid.height as i32 {
                
                let cell_center = nav_grid.grid_to_world(GridNode::new(cell_x as u32, cell_z as u32));
                let distance = center.distance(cell_center);
                
                if distance <= radius {
                    set_cell_walkable(nav_grid, GridNode::new(cell_x as u32, cell_z as u32), false);
                }
            }
        }
    }
}
```

**Rectangular Area Blocking** (for default objects):
```rust
fn block_rectangular_area(nav_grid: &mut NavigationGrid, center: Vec3, half_extents: Vec3) {
    let min_world = center - half_extents;
    let max_world = center + half_extents;
    
    let min_cell = nav_grid.world_to_grid(min_world)?;
    let max_cell = nav_grid.world_to_grid(max_world)?;
    
    for z in min_cell.z..=max_cell.z {
        for x in min_cell.x..=max_cell.x {
            set_cell_walkable(nav_grid, GridNode::new(x, z), false);
        }
    }
}
```

### 3. Modified Navigation Grid Creation

**New Signature**:
```rust
impl NavigationGrid {
    pub fn from_terrain_and_objects(
        terrain: &TerrainData, 
        objects: &[EnvironmentObject],
        config: PathfindingConfig
    ) -> MinionResult<Self> {
        // 1. Build terrain-based walkability (existing logic)
        let mut nav_grid = Self::from_terrain(terrain, config)?;
        
        // 2. Apply environment object blocking
        for obj in objects {
            project_object_to_grid(obj, &mut nav_grid);
        }
        
        Ok(nav_grid)
    }
}
```

### 4. Integration Point Changes

**Map Loader Update** (`src/plugins/map_loader.rs`):
```rust
// Replace lines 25-30, 51-56, 73-78 with:
match NavigationGrid::from_terrain_and_objects(&map.terrain, &map.environment_objects, pathfinding_config) {
    Ok(nav_grid) => {
        info!("Successfully created navigation grid with {obj_count} environment objects", 
              obj_count = map.environment_objects.len());
        commands.insert_resource(nav_grid);
    }
    Err(err) => {
        warn!("Failed to create navigation grid: {err}");
        warn!("Pathfinding will not be available - falling back to direct movement");
    }
}
```

## Performance Analysis

### Computational Complexity

**Grid Building**: O(G + O*A) where:
- G = grid cells (4096 for 64x64)  
- O = environment objects (~400 typical)
- A = average cells per object (~9 for 1.5 unit radius objects)

**Total**: ~4096 + 400*9 = **~7696 operations** per map load

**Memory Impact**: No additional memory - uses existing `walkable: Vec<bool>`

### Benchmarking Estimates

- **64x64 grid**: ~0.1ms terrain processing  
- **400 objects**: ~0.3ms object projection
- **Total grid building**: **~0.4ms** (negligible impact)

### Optimization Opportunities

1. **Early Culling**: Skip objects outside terrain bounds
2. **Size Filtering**: Skip tiny objects (grass) that don't meaningfully block paths
3. **Batch Processing**: Group objects by type for optimized projection
4. **Grid Pre-allocation**: Reserve space for walkability vector

## Error Handling & Edge Cases

### Robust Error Handling
```rust
fn project_object_to_grid(obj: &EnvironmentObject, nav_grid: &mut NavigationGrid) -> MinionResult<()> {
    // 1. Validate object is within terrain bounds
    if !nav_grid.is_position_in_bounds(obj.position) {
        debug!("Skipping object {} outside terrain bounds at {:?}", obj.object_type, obj.position);
        return Ok(());
    }
    
    // 2. Handle zero/negative scales gracefully
    if obj.scale.x <= 0.0 || obj.scale.z <= 0.0 {
        warn!("Invalid object scale {:?} for {}, skipping", obj.scale, obj.object_type);
        return Ok(());
    }
    
    // 3. Apply object-specific projection...
    
    Ok(())
}
```

### Edge Case Handling
- **Objects outside terrain**: Skip with debug log
- **Invalid scales**: Skip with warning
- **Unknown object types**: Use default cuboid projection
- **Overlapping objects**: Multiple blocking calls are safe (idempotent)
- **Complete area blocking**: Ensure at least one path exists to player spawn

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    #[test]  
    fn test_navigation_grid_with_objects() {
        let terrain = TerrainData::create_flat(8, 8, 1.0, 0.0).unwrap();
        let objects = vec![
            EnvironmentObject::simple("rock".to_string(), Vec3::new(2.0, 0.0, 2.0)),
        ];
        
        let nav_grid = NavigationGrid::from_terrain_and_objects(&terrain, &objects, PathfindingConfig::default()).unwrap();
        
        // Rock should block its grid cell
        let rock_cell = nav_grid.world_to_grid(Vec3::new(2.0, 0.0, 2.0)).unwrap();
        assert!(!nav_grid.is_walkable(rock_cell.x, rock_cell.z));
        
        // Adjacent cells should remain walkable
        assert!(nav_grid.is_walkable(rock_cell.x + 1, rock_cell.z));
    }
    
    #[test]
    fn test_circular_blocking_area() {
        // Test that circular objects block appropriate cells
    }
    
    #[test]
    fn test_object_outside_terrain_bounds() {
        // Test graceful handling of out-of-bounds objects
    }
}
```

### Integration Tests
- Full map loading with varied object layouts
- Pathfinding through object-dense areas  
- Performance benchmarking with 400+ objects

## Implementation Roadmap

### Phase 1: Core Integration (1-2 hours)
1. **Modify NavigationGrid::from_terrain()** → `from_terrain_and_objects()`
2. **Implement geometric projection functions**
3. **Update map loader integration points**
4. **Add basic error handling**

### Phase 2: Optimization & Polish (1 hour)  
1. **Add object type specific blocking logic**
2. **Implement bounds checking and edge case handling**
3. **Add debug logging for blocked cells**

### Phase 3: Testing & Validation (1 hour)
1. **Write comprehensive unit tests**
2. **Test with existing maps and object layouts**
3. **Validate pathfinding behavior in object-dense areas**

**Total Estimated Time**: **3-4 hours** for complete implementation

## Future Extensions

### Dynamic Object Support
- **Framework Ready**: Object projection functions can be called at runtime
- **Use Cases**: Destructible environment, spawned obstacles, moving platforms
- **API**: `nav_grid.update_object_blocking(obj, is_blocking: bool)`

### Advanced Blocking Logic
- **Partial Blocking**: Height-based walkability (low walls vs tall walls)
- **Directional Blocking**: Objects that block some movement directions
- **Cost Modifiers**: Objects that increase movement cost instead of blocking

### Performance Enhancements
- **Spatial Indexing**: Grid-based object lookup for large object counts
- **Incremental Updates**: Only recompute affected grid regions
- **Multi-threading**: Parallel object projection for massive maps

## Conclusion

This design provides an elegant, maintainable solution for integrating environment objects into the pathfinding navigation grid. The approach:

- ✅ **Solves the core problem**: Players won't get stuck on rocks/trees
- ✅ **Maintains clean architecture**: Clear separation between data and ECS
- ✅ **Optimizes performance**: Single-pass grid building with minimal overhead  
- ✅ **Enables extensibility**: Framework ready for dynamic objects and advanced features
- ✅ **Ensures reliability**: Comprehensive error handling and testing strategy

The implementation is straightforward, testable, and follows the existing codebase patterns while providing immediate value and a foundation for future enhancements.