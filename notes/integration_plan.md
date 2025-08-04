# Bevy/Rapier3D Integration Plan

## Overview

This plan details how to integrate the sophisticated biome-based terrain generation system with the existing Bevy ECS and Rapier3D physics systems while maintaining performance and architectural cleanliness.

## Bevy ECS Integration

### Component Architecture

#### Core Components
```rust
// Existing component - no changes needed
#[derive(Component)]
pub struct TerrainMesh {
    pub collider: Handle<Collider>,
}

// New biome-aware components
#[derive(Component)]
pub struct BiomeTerrainMesh {
    pub base_terrain: Handle<TerrainMesh>,
    pub biome_data: Handle<BiomeTerrainData>,
    pub material_handles: HashMap<SurfaceType, Handle<StandardMaterial>>,
}

#[derive(Component)]
pub struct BiomeRegionMarker {
    pub biome_type: BiomeType,
    pub region_id: u32,
    pub bounds: Rect,
}

#[derive(Component)]
pub struct PathNode {
    pub node_id: u32,
    pub connections: Vec<u32>,
    pub path_type: PathType,
    pub world_position: Vec3,
}

#[derive(Component)]
pub struct PathEdge {
    pub from_node: u32,
    pub to_node: u32,
    pub spline_points: Vec<Vec3>,
    pub surface_type: SurfaceType,
}

#[derive(Component)]
pub struct BiomeEnvironmentObject {
    pub base_object: EnvironmentObject, // Existing structure
    pub size_category: ObjectSizeCategory,
    pub biome_source: BiomeType,
    pub placement_priority: f32, // For LOD
}
```

#### Resource Extensions
```rust
// Extend existing terrain resources
#[derive(Resource)]
pub struct BiomeTerrainConfig {
    pub enabled: bool,
    pub region_count: u32,
    pub transition_sharpness: f32,
    pub path_generation_enabled: bool,
    pub object_density_multiplier: f32,
}

#[derive(Resource)]
pub struct BiomeAssets {
    pub materials: HashMap<SurfaceType, Handle<StandardMaterial>>,
    pub object_meshes: HashMap<(BiomeType, ObjectSizeCategory), Vec<Handle<Mesh>>>,
    pub path_materials: HashMap<PathType, Handle<StandardMaterial>>,
}
```

### System Architecture

#### Generation Systems (Run during map loading)
```rust
// Extends existing terrain generation pipeline
pub fn generate_biome_terrain_system(
    mut commands: Commands,
    terrain_config: Res<BiomeTerrainConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    map_definition: Res<MapDefinition>,
) {
    // 1. Generate BiomeTerrainData from existing TerrainData
    // 2. Create multi-material mesh with biome blending
    // 3. Generate path network entities
    // 4. Place biome-appropriate environment objects
    // 5. Create spatial index for runtime queries
}

pub fn biome_object_placement_system(
    mut commands: Commands,
    biome_terrain: Res<BiomeTerrainData>,
    biome_assets: Res<BiomeAssets>,
    config: Res<BiomeTerrainConfig>,
) {
    // 1. Calculate placement points using Poisson disk sampling
    // 2. Assign object types based on biome blend weights
    // 3. Apply size variation and clustering rules
    // 4. Respect path network constraints
    // 5. Create entities with LOD components
}
```

#### Runtime Systems
```rust
pub fn biome_query_system(
    terrain_query: Query<&BiomeTerrainMesh>,
    player_query: Query<&Transform, With<Player>>,
    mut query_cache: ResMut<BiomeQueryCache>,
) {
    // Fast biome lookups for gameplay systems
    // Cache frequently queried positions
    // Update player's current biome information
}

pub fn path_navigation_system(
    path_nodes: Query<&PathNode>,
    path_edges: Query<&PathEdge>,
    mut navigation_requests: EventReader<NavigationRequest>,
    mut navigation_responses: EventWriter<NavigationResponse>,
) {
    // Handle pathfinding requests using pregenerated network
    // Provide natural walking paths for AI entities
    // Update path costs based on dynamic factors
}

pub fn biome_object_lod_system(
    mut object_query: Query<(&BiomeEnvironmentObject, &mut Visibility, &Transform)>,
    camera_query: Query<&Transform, (With<Camera>, Without<BiomeEnvironmentObject>)>,
    config: Res<BiomeTerrainConfig>,
) {
    // Cull objects based on distance and size category
    // Implement hierarchical LOD for dense object placement
    // Prioritize larger objects for distant viewing
}
```

### Event Architecture
```rust
#[derive(Event)]
pub struct BiomeTransitionEvent {
    pub entity: Entity,
    pub from_biome: BiomeType,
    pub to_biome: BiomeType,
    pub transition_strength: f32,
}

#[derive(Event)]
pub struct NavigationRequest {
    pub entity: Entity,
    pub from: Vec3,
    pub to: Vec3,
    pub movement_type: MovementType,
}

#[derive(Event)]
pub struct NavigationResponse {
    pub entity: Entity,
    pub path: Option<Vec<Vec3>>,
    pub total_cost: f32,
}
```

## Rapier3D Physics Integration

### Collider Generation Strategy

#### Multi-Material Terrain Colliders
```rust
pub fn generate_biome_terrain_collider(
    biome_terrain: &BiomeTerrainData
) -> MinionResult<Collider> {
    // 1. Use existing terrain mesh generation as base
    // 2. Apply surface-type specific physics properties
    // 3. Generate separate colliders for different material zones
    // 4. Combine using compound collider for efficiency
    
    let base_collider = generate_terrain_collider(&biome_terrain.base)?;
    
    // Add material-specific collision properties
    let mut compound_collider_data = Vec::new();
    
    // Water zones - sensor colliders for swimming detection
    for water_zone in find_water_zones(biome_terrain) {
        let water_collider = generate_zone_collider(water_zone)?;
        compound_collider_data.push((
            Isometry::identity(),
            water_collider.with_sensor(true)
        ));
    }
    
    // Ice zones - low friction colliders
    for ice_zone in find_ice_zones(biome_terrain) {
        let ice_collider = generate_zone_collider(ice_zone)?;
        compound_collider_data.push((
            Isometry::identity(),
            ice_collider.with_friction(0.1)
        ));
    }
    
    Ok(Collider::compound(compound_collider_data))
}
```

#### Path Colliders
```rust
pub fn generate_path_colliders(
    path_network: &PathNetwork
) -> Vec<(Transform, Collider)> {
    path_network.paths.iter().map(|path| {
        // Create capsule colliders along path splines
        // Lower friction for easier movement
        // Slightly elevated to avoid terrain conflicts
        let collider = Collider::capsule_y(0.1, path.width / 2.0)
            .with_friction(0.3)
            .with_restitution(0.0);
        
        (path.transform, collider)
    }).collect()
}
```

#### Object Colliders by Size
```rust
pub fn create_object_collider(
    object: &BiomeEnvironmentObject
) -> Collider {
    match object.size_category {
        ObjectSizeCategory::Pebbles => {
            // No collider - just visual
            Collider::ball(0.0).with_sensor(true)
        },
        ObjectSizeCategory::SmallRocks => {
            Collider::ball(0.2).with_mass(1.0)
        },
        ObjectSizeCategory::MediumRocks => {
            Collider::ball(0.5).with_mass(5.0)
        },
        ObjectSizeCategory::LargeRocks => {
            Collider::ball(1.0).with_mass(20.0)
        },
        ObjectSizeCategory::Boulders => {
            Collider::ball(2.0).with_mass(100.0)
        },
        ObjectSizeCategory::Trees => {
            // Cylinder for tree trunk
            Collider::cylinder(3.0, 0.3).with_mass(50.0)
        },
    }
}
```

### Physics Materials by Surface Type
```rust
#[derive(Resource)]
pub struct BiomePhysicsMaterials {
    pub materials: HashMap<SurfaceType, PhysicsMaterial>,
}

impl Default for BiomePhysicsMaterials {
    fn default() -> Self {
        let mut materials = HashMap::new();
        
        materials.insert(SurfaceType::Grass, PhysicsMaterial {
            friction: 0.7,
            restitution: 0.1,
            ..default()
        });
        
        materials.insert(SurfaceType::Rock(_), PhysicsMaterial {
            friction: 0.9,
            restitution: 0.3,
            ..default()
        });
        
        materials.insert(SurfaceType::Ice, PhysicsMaterial {
            friction: 0.1,
            restitution: 0.0,
            ..default()
        });
        
        materials.insert(SurfaceType::Sand, PhysicsMaterial {
            friction: 0.8,
            restitution: 0.05,
            ..default()
        });
        
        materials.insert(SurfaceType::Water, PhysicsMaterial {
            friction: 0.0,
            restitution: 0.0,
            ..default()
        });
        
        Self { materials }
    }
}
```

### Movement Integration
```rust
pub fn biome_movement_system(
    mut player_query: Query<(&mut ExternalForce, &Transform), With<Player>>,
    biome_terrain: Res<BiomeTerrainData>,
    physics_materials: Res<BiomePhysicsMaterials>,
) {
    for (mut force, transform) in player_query.iter_mut() {
        // Query surface type at player position
        if let Some(surface_type) = get_surface_at_position(
            &biome_terrain, 
            transform.translation.xz()
        ) {
            // Apply surface-specific movement modifiers
            match surface_type {
                SurfaceType::Ice => {
                    // Reduce control on ice
                    force.force *= 0.3;
                },
                SurfaceType::Sand => {
                    // Slower movement in sand
                    force.force *= 0.7;
                },
                SurfaceType::Water => {
                    // Swimming mechanics
                    force.force *= 0.5;
                    // Add buoyancy force
                    force.force.y += 20.0;
                },
                _ => {
                    // Normal movement
                }
            }
        }
    }
}
```

## Rendering Integration

### Multi-Material Terrain Rendering
```rust
pub fn setup_biome_terrain_materials(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
) {
    let mut biome_materials = HashMap::new();
    
    // Load texture arrays for each surface type
    biome_materials.insert(SurfaceType::Grass, materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load("textures/grass_diffuse.png")),
        normal_map_texture: Some(asset_server.load("textures/grass_normal.png")),
        ..default()
    }));
    
    biome_materials.insert(SurfaceType::Rock(RockSize::MediumRocks), materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load("textures/rock_diffuse.png")),
        normal_map_texture: Some(asset_server.load("textures/rock_normal.png")),
        metallic: 0.0,
        roughness: 0.9,
        ..default()
    }));
    
    // Additional materials...
    
    commands.insert_resource(BiomeAssets {
        materials: biome_materials,
        ..default()
    });
}
```

### Shader Integration (Advanced)
For maximum visual quality, consider custom shaders that:
- Blend multiple textures based on biome weights
- Apply detail textures based on surface type
- Use height-based texture transitions
- Implement triplanar mapping for steep surfaces

## Performance Considerations

### Spatial Partitioning
```rust
#[derive(Resource)]
pub struct BiomeSpatialIndex {
    pub biome_grid: Vec<Vec<BiomeCell>>,
    pub cell_size: f32,
    pub bounds: Rect,
}

impl BiomeSpatialIndex {
    pub fn query_biome(&self, world_pos: Vec2) -> Option<&BiomeCell> {
        // O(1) biome lookup using spatial grid
        // Cache blend weights for frequently queried cells
    }
    
    pub fn query_objects_in_radius(&self, center: Vec2, radius: f32) -> Vec<Entity> {
        // Efficient object queries for gameplay systems
        // Use spatial hashing for collision detection
    }
}
```

### Level of Detail Strategy
1. **Distance-based culling**: Hide small objects beyond view distance
2. **Size-based prioritization**: Render large objects first
3. **Biome-based LOD**: Different detail levels per biome type
4. **Dynamic mesh resolution**: Reduce terrain vertices at distance

### Memory Management
1. **Streaming**: Load biome data in chunks for large worlds
2. **Compression**: Use texture atlases for material efficiency
3. **Pooling**: Reuse object entities for dynamic placement
4. **Caching**: Store computed values for expensive operations

## Error Handling Integration

### Graceful Degradation
```rust
pub fn safe_biome_terrain_generation(
    terrain_data: &TerrainData,
    config: &BiomeTerrainConfig,
) -> MinionResult<BiomeTerrainData> {
    // Fall back to single-biome terrain if generation fails
    BiomeTerrainData::generate(terrain_data, config)
        .or_else(|_| {
            warn!("Biome generation failed, falling back to grassland");
            BiomeTerrainData::fallback_grassland(terrain_data)
        })
}
```

### Validation Systems
```rust
pub fn validate_biome_terrain_system(
    biome_terrain_query: Query<&BiomeTerrainMesh>,
    mut diagnostics: ResMut<BiomeDiagnostics>,
) {
    for biome_terrain in biome_terrain_query.iter() {
        // Validate biome weight normalization
        // Check for unreachable path nodes
        // Verify object placement constraints
        // Report performance metrics
    }
}
```

## Migration Strategy

### Phase 1: Foundation (Immediate)
1. Add new components alongside existing terrain system
2. Implement basic biome data structures
3. Create conversion utilities for existing maps
4. No changes to existing gameplay

### Phase 2: Core Integration (Week 1-2)
1. Implement biome-aware mesh generation
2. Add physics material integration
3. Basic object placement with size variation
4. Performance optimization for 1024x1024 terrain

### Phase 3: Advanced Features (Week 3-4)
1. Path network generation and navigation
2. Advanced biome blending
3. Runtime biome queries for gameplay
4. Visual improvements and shader integration

### Phase 4: Polish (Week 5+)
1. Tool integration for biome editing
2. Performance profiling and optimization
3. Documentation and examples
4. Advanced biome types and interactions

This integration plan provides a comprehensive roadmap for incorporating sophisticated terrain generation while maintaining the existing system's performance characteristics and architectural principles.