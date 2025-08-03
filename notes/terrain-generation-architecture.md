# Procedural Terrain Generation Architecture

## Overview
Stage 6 architecture for adding noise-based terrain generation to the mapgen binary, building on existing TerrainData and mesh generation systems.

## System Architecture

### Core Components

#### 1. Terrain Generator Module
**Location:** `src/terrain/generator.rs`
**Purpose:** High-level terrain generation with multiple algorithms

```rust
pub struct TerrainGenerator {
    pub seed: u32,
    pub algorithm: TerrainAlgorithm,
}

pub enum TerrainAlgorithm {
    Flat { height: f32 },
    Perlin { 
        amplitude: f32, 
        frequency: f32, 
        octaves: u32,
        persistence: f32,
        lacunarity: f32,
    },
    Ridged { 
        amplitude: f32, 
        frequency: f32, 
        octaves: u32,
    },
    Layered {
        base: Box<TerrainAlgorithm>,
        detail: Box<TerrainAlgorithm>,
        detail_weight: f32,
    },
}

impl TerrainGenerator {
    pub fn generate(&self, width: u32, height: u32, scale: f32) -> MinionResult<TerrainData>;
}
```

#### 2. Biome System
**Location:** `src/terrain/biomes.rs`
**Purpose:** Terrain type variations and spawn zone intelligence

```rust
pub struct BiomeMap {
    pub moisture_map: Vec<f32>,
    pub temperature_map: Vec<f32>,
    pub biomes: Vec<BiomeType>,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub enum BiomeType {
    Plains,      // Low hills, good spawning
    Hills,       // Medium elevation, moderate spawning
    Mountains,   // High elevation, limited spawning
    Water,       // Below water level, no spawning
    Swamp,       // Low elevation + high moisture
}

impl BiomeMap {
    pub fn generate(terrain: &TerrainData, water_level: f32) -> Self;
    pub fn get_biome_at(&self, x: u32, y: u32) -> BiomeType;
    pub fn is_suitable_for_spawning(&self, x: u32, y: u32) -> bool;
}
```

#### 3. Spawn Zone Intelligence
**Location:** `src/terrain/spawn_placement.rs`
**Purpose:** Terrain-aware spawn zone placement

```rust
pub struct SpawnPlacement {
    terrain: TerrainData,
    biome_map: BiomeMap,
    water_level: f32,
    max_slope: f32,  // Maximum terrain slope for spawning
}

impl SpawnPlacement {
    pub fn find_spawn_locations(&self, num_zones: u32, player_spawn: Vec3) -> MinionResult<Vec<Vec3>>;
    pub fn validate_spawn_location(&self, position: Vec3) -> bool;
    pub fn adjust_spawn_to_terrain(&self, position: Vec3) -> Vec3;
}
```

### Integration with Existing System

#### Enhanced MapGen CLI
**Location:** `src/bin/mapgen.rs`
**New parameters:**

```rust
#[derive(Parser)]
struct Args {
    // ... existing fields ...
    
    /// Terrain generation algorithm
    #[arg(long, default_value = "perlin")]
    terrain_type: String,
    
    /// Random seed for generation
    #[arg(long)]
    seed: Option<u32>,
    
    /// Terrain amplitude (height variation)
    #[arg(long, default_value = "10.0")]
    amplitude: f32,
    
    /// Base frequency for noise
    #[arg(long, default_value = "0.02")]
    frequency: f32,
    
    /// Number of noise octaves
    #[arg(long, default_value = "4")]
    octaves: u32,
    
    /// Water level (affects biome generation)
    #[arg(long, default_value = "0.0")]
    water_level: f32,
    
    /// Enable terrain presets
    #[arg(long)]
    preset: Option<String>,
}
```

#### Terrain Presets
Pre-configured terrain types for ease of use:

```rust
pub fn get_terrain_preset(name: &str) -> Option<TerrainGenerator> {
    match name {
        "flat" => Some(TerrainGenerator::flat(0.0)),
        "rolling" => Some(TerrainGenerator::perlin(5.0, 0.03, 3)),
        "hills" => Some(TerrainGenerator::perlin(15.0, 0.02, 4)),
        "mountains" => Some(TerrainGenerator::perlin(30.0, 0.015, 5)),
        "archipelago" => Some(TerrainGenerator::layered_with_water()),
        _ => None,
    }
}
```

## Spawn Zone Intelligence Strategy

### 1. Terrain Analysis
**Slope Calculation:** Use height differences between adjacent grid points
```rust
fn calculate_slope(terrain: &TerrainData, x: u32, y: u32) -> f32 {
    let h_center = terrain.get_height_at_grid(x, y).unwrap_or(0.0);
    let h_right = terrain.get_height_at_grid(x + 1, y).unwrap_or(h_center);
    let h_up = terrain.get_height_at_grid(x, y + 1).unwrap_or(h_center);
    
    let dx = (h_right - h_center) / terrain.scale;
    let dy = (h_up - h_center) / terrain.scale;
    
    (dx * dx + dy * dy).sqrt()
}
```

### 2. Accessibility Scoring
**Multi-factor spawn suitability:**
- **Slope:** < 0.3 radians (gentle slopes only)
- **Height:** Above water level, below extreme elevations
- **Distance:** Minimum separation from other spawn zones
- **Accessibility:** Pathfinding validation to player spawn

### 3. Spawn Zone Placement Algorithm
```rust
fn place_spawn_zones(
    terrain: &TerrainData,
    biome_map: &BiomeMap,
    player_spawn: Vec3,
    num_zones: u32,
) -> MinionResult<Vec<SpawnZone>> {
    let mut zones = Vec::new();
    let mut attempts = 0;
    const MAX_ATTEMPTS: u32 = 1000;
    
    while zones.len() < num_zones as usize && attempts < MAX_ATTEMPTS {
        let candidate = generate_candidate_position(player_spawn, zones.len());
        
        if is_suitable_spawn_location(terrain, biome_map, candidate) {
            let zone = create_spawn_zone_at(candidate, terrain);
            zones.push(zone);
        }
        
        attempts += 1;
    }
    
    Ok(zones)
}
```

## Performance Considerations

### 1. Generation Caching
**Noise Function Reuse:** Cache noise generators between terrain chunks
**Height Map Streaming:** Generate large terrains in tiles

### 2. Memory Management
**Lazy Loading:** Generate biome maps only when spawn placement needed
**Temporary Allocations:** Use stack allocation for small noise calculations

### 3. Scalability Limits
**Target Performance:**
- 64x64 terrain: < 100ms generation
- 256x256 terrain: < 2s generation
- 512x512 terrain: < 10s generation (warning threshold)

## Error Handling Strategy

### 1. Graceful Degradation
**Noise Generation Failure:** Fall back to simpler noise or flat terrain
**Spawn Placement Failure:** Reduce requirements (allow steeper slopes)
**Biome Generation Issues:** Use elevation-only classification

### 2. Validation Pipeline
```rust
pub fn validate_generated_terrain(terrain: &TerrainData) -> MinionResult<()> {
    // Check height bounds
    let (min_h, max_h) = terrain.height_bounds();
    if max_h - min_h > 100.0 {
        return Err(MinionError::InvalidMapData { 
            reason: "Terrain height variation too extreme".to_string()
        });
    }
    
    // Validate no NaN or infinite values
    if terrain.heights.iter().any(|&h| !h.is_finite()) {
        return Err(MinionError::InvalidMapData {
            reason: "Invalid height values detected".to_string()
        });
    }
    
    Ok(())
}
```

## Implementation Phases

### Phase 1: Basic Noise Integration (Week 1)
1. Add noise-functions dependency
2. Implement TerrainGenerator with Perlin noise
3. Extend mapgen CLI with basic parameters
4. Update terrain generation in mapgen binary

### Phase 2: Biome System (Week 2)
1. Implement BiomeMap generation
2. Add biome-based terrain coloring/texturing support
3. Create terrain presets

### Phase 3: Intelligent Spawn Placement (Week 3)
1. Implement SpawnPlacement system
2. Add slope and accessibility validation
3. Update spawn zone generation algorithm

### Phase 4: Polish and Optimization (Week 4)
1. Performance optimization
2. Error handling improvements
3. Additional terrain presets
4. Documentation and examples

## Testing Strategy

### 1. Unit Tests
- Noise generation consistency
- Biome classification accuracy
- Spawn placement validation

### 2. Integration Tests
- End-to-end mapgen generation
- Terrain mesh/collider compatibility
- Game loading of procedural maps

### 3. Performance Tests
- Generation time benchmarks
- Memory usage profiling
- Large terrain stress tests

This architecture provides a solid foundation for procedural terrain generation while maintaining compatibility with existing systems and following the project's minimalist principles.