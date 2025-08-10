# Sophisticated Terrain Generation Architecture

## Overview

This design extends the existing noise-based terrain system with sophisticated biome transitions, multiple surface types, intelligent path generation, and varied object placement while maintaining the project's minimalist philosophy and backward compatibility.

## Core Architecture

### 1. Layered Generation System

The terrain generation follows a layered approach where each layer builds upon the previous:

```
Base Height Layer (existing)     → Heightmap via noise functions
↓
Biome Region Layer (new)         → Voronoi regions with climate data
↓
Biome Blending Layer (new)       → Smooth transitions between regions
↓
Surface Material Layer (new)     → Material assignment per biome
↓
Path Network Layer (new)         → Natural walking paths
↓
Object Placement Layer (enhanced) → Size-varied, biome-appropriate objects
```

### 2. Extended Data Structures

#### BiomeTerrainData (extends TerrainData)
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct BiomeTerrainData {
    // Existing terrain data
    pub base: TerrainData,

    // New biome data
    pub biome_map: Vec<BiomeType>,     // Per-vertex primary biome
    pub blend_weights: Vec<[f32; 4]>,  // Up to 4 biomes per vertex with weights
    pub surface_materials: Vec<SurfaceType>, // Surface material per vertex
    pub path_network: PathNetwork,     // Generated path data

    // Generation metadata
    pub biome_seed: u32,
    pub biome_regions: Vec<BiomeRegion>,
}
```

#### BiomeRegion
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeRegion {
    pub biome_type: BiomeType,
    pub center: Vec2,              // Voronoi seed point
    pub temperature: f32,          // 0.0 (cold) to 1.0 (hot)
    pub humidity: f32,             // 0.0 (dry) to 1.0 (wet)
    pub elevation_bias: f32,       // Height modifier for this biome
    pub transition_radius: f32,    // Blend distance at edges
}
```

#### BiomeType and SurfaceType
```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum BiomeType {
    Grassland,
    Forest,
    Mountains,
    Desert,
    Tundra,
    Swamp,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SurfaceType {
    Grass,
    Dirt,
    Rock(RockSize),
    Sand,
    Ice,
    Water,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RockSize {
    Pebbles,
    SmallRocks,
    MediumRocks,
    LargeRocks,
    Boulders,
}
```

### 3. Generation Pipeline

#### Stage 1: Base Height Generation (Existing)
- Use current noise-based system for base terrain height
- Maintain all existing terrain presets and algorithms
- No changes to existing `TerrainGenerator`

#### Stage 2: Biome Region Generation
```rust
pub struct BiomeGenerator {
    pub seed: u32,
    pub region_count: u32,          // Number of Voronoi regions
    pub temperature_noise: PerlinConfig,
    pub humidity_noise: PerlinConfig,
    pub region_size_bias: f32,      // Controls size variation
}

impl BiomeGenerator {
    pub fn generate_regions(&self, bounds: Rect) -> Vec<BiomeRegion> {
        // 1. Generate Voronoi seed points using blue noise distribution
        // 2. Assign temperature/humidity using noise functions
        // 3. Determine biome type from climate data
        // 4. Calculate transition radii based on region spacing
    }
}
```

#### Stage 3: Biome Blending
```rust
pub struct BiomeBlender {
    pub transition_sharpness: f32,  // Controls blend falloff
    pub max_blend_distance: f32,    // Maximum blend radius
}

impl BiomeBlender {
    pub fn calculate_weights(&self,
        position: Vec2,
        regions: &[BiomeRegion]) -> BlendWeights {
        // 1. Find nearest 4 biome regions using Voronoi distance
        // 2. Calculate distance-based weights with smooth falloff
        // 3. Normalize weights to sum to 1.0
        // 4. Apply transition sharpness curve
    }
}
```

#### Stage 4: Surface Material Assignment
```rust
pub struct SurfaceMaterializer {
    pub rock_size_noise: RidgedConfig,
    pub material_transition_noise: PerlinConfig,
}

impl SurfaceMaterializer {
    pub fn assign_materials(&self,
        biome_weights: &BlendWeights,
        height: f32,
        slope: f32,
        position: Vec2) -> SurfaceType {
        // 1. Primary material from dominant biome
        // 2. Height-based overrides (rock on peaks, water in valleys)
        // 3. Slope-based overrides (rock on steep areas)
        // 4. Noise-based variation within biome rules
        // 5. Rock size variation using additional noise
    }
}
```

#### Stage 5: Path Network Generation
```rust
pub struct PathNetwork {
    pub nodes: Vec<PathNode>,
    pub edges: Vec<PathEdge>,
    pub paths: Vec<GeneratedPath>,
}

pub struct PathGenerator {
    pub node_density: f32,          // Nodes per world unit
    pub path_preference: PathPreference,
    pub min_path_length: f32,
    pub max_path_length: f32,
}

impl PathGenerator {
    pub fn generate_paths(&self,
        terrain: &BiomeTerrainData,
        interesting_points: &[Vec2]) -> PathNetwork {
        // 1. Generate potential path nodes using spatial sampling
        // 2. Filter nodes by terrain suitability (slope, surface type)
        // 3. Connect nodes using A* with terrain-aware cost function
        // 4. Optimize paths for natural appearance
        // 5. Add branching and alternative routes
    }
}
```

#### Stage 6: Enhanced Object Placement
```rust
pub struct BiomeObjectPlacer {
    pub density_per_biome: HashMap<BiomeType, f32>,
    pub size_distribution: SizeDistribution,
    pub clustering_factor: f32,
}

impl BiomeObjectPlacer {
    pub fn place_objects(&self,
        terrain: &BiomeTerrainData) -> Vec<EnvironmentObject> {
        // 1. Calculate placement density from biome blend weights
        // 2. Use Poisson disk sampling for natural distribution
        // 3. Apply size variation based on clustering rules
        // 4. Respect path network (avoid placing on paths)
        // 5. Add size-appropriate object types per biome
    }
}
```

## Library Integration

### Dependencies to Add
```toml
[dependencies]
# Existing dependencies remain unchanged
voronoice = "3.0"          # Voronoi diagrams for biome regions
pathfinding = "4.10"       # A* and other pathfinding algorithms
```

### Core Integration Points

#### 1. Extended TerrainGenerator
```rust
pub enum TerrainAlgorithm {
    // Existing variants unchanged
    Flat { height: f32 },
    Perlin { amplitude: f32, frequency: f32, octaves: u32 },
    Ridged { amplitude: f32, frequency: f32, octaves: u32 },

    // New biome-based variant
    BiomeBased {
        base_algorithm: Box<TerrainAlgorithm>,
        biome_config: BiomeGenerationConfig,
    },
}
```

#### 2. Backward-Compatible Data Loading
```rust
impl TerrainData {
    pub fn upgrade_to_biome_terrain(&self) -> BiomeTerrainData {
        // Convert existing terrain to biome-based with default biome
        // Maintains compatibility with existing saved maps
    }
}
```

## Performance Optimizations

### 1. Spatial Indexing
- Use spatial hash grids for fast biome region lookups
- Cache Voronoi cell assignments for frequently queried areas
- Precompute blend weight lookup tables for common distances

### 2. Lazy Evaluation
- Generate surface materials on-demand during mesh creation
- Stream path generation for large terrains
- Level-of-detail for object placement based on view distance

### 3. Memory Efficiency
- Store blend weights only for transition zones
- Use bit packing for surface type storage
- Compress path data using simplified splines

## Terrain Features Implementation

### Gradual Biome Transitions
- **Method**: Distance-based weight blending using smooth curves
- **Function**: `smoothstep()` for natural falloff
- **Range**: Configurable transition radius per biome (default 50-200 world units)

### Multiple Surface Types
- **Rock Sizes**: Controlled by fractal noise with octaves for detail
- **Placement Logic**: Height and slope thresholds per biome
- **Variation**: Secondary noise for realistic distribution

### Intelligent Path Generation
- **Cost Function**: Considers slope, surface type, and existing paths
- **Natural Curves**: Post-process straight A* paths with spline smoothing
- **Branching**: Generate tree-like path networks from major nodes

### Size Variation
- **Distribution**: Log-normal for realistic size spread
- **Clustering**: Group small objects near large ones
- **Biome Rules**: Different size preferences per biome type

## Integration with Existing Systems

### Bevy ECS Components
```rust
#[derive(Component)]
pub struct BiomeInfo {
    pub primary_biome: BiomeType,
    pub blend_weights: BlendWeights,
    pub surface_type: SurfaceType,
}

#[derive(Component)]
pub struct PathNode {
    pub node_id: u32,
    pub connections: Vec<u32>,
    pub path_type: PathType,
}
```

### Rapier3D Physics
- Generate physics colliders from surface types
- Different friction values per surface material
- Path areas use lower friction for easier movement

### Rendering Pipeline
- Multi-texture blending based on biome weights
- Normal map variation per surface type
- Path rendering with unique materials

## Error Handling Strategy

Following the project's error handling patterns:

```rust
#[derive(Error, Debug)]
pub enum BiomeGenerationError {
    #[error("Invalid biome configuration: {reason}")]
    InvalidBiomeConfig { reason: String },

    #[error("Voronoi generation failed: {reason}")]
    VoronoiGenerationFailed { reason: String },

    #[error("Path generation failed: {reason}")]
    PathGenerationFailed { reason: String },
}
```

## Testing Strategy

### Unit Tests
- Biome region generation with known seeds
- Blend weight calculations for edge cases
- Path generation cost function validation
- Surface material assignment logic

### Integration Tests
- Full terrain generation pipeline
- Backward compatibility with existing maps
- Performance benchmarks for 1024x1024 terrain
- Serialization round-trip tests

### Property-Based Tests
- Biome weight normalization (always sum to 1.0)
- Path connectivity validation
- Object placement constraints

## Migration Path

### Phase 1: Foundation
1. Add new data structures alongside existing ones
2. Implement basic biome region generation
3. Create conversion utilities for existing maps

### Phase 2: Core Features
1. Implement biome blending system
2. Add surface material assignment
3. Enhanced object placement with size variation

### Phase 3: Advanced Features
1. Path network generation
2. Performance optimizations
3. Advanced biome types and transitions

### Phase 4: Polish
1. Tool integration for map editing
2. Runtime configuration options
3. Documentation and examples

This architecture provides a sophisticated yet maintainable extension to the existing terrain system, following the project's minimalist philosophy while achieving the ambitious goals of gradual biome transitions, multiple surface types, intelligent path generation, and varied object placement.
