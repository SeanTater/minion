# MapGen CLI Design Specification

## Philosophy
Design CLI following Unix principles: simple defaults, powerful when needed, composable parameters, clear error messages. Prioritize ease of use for non-experts while providing full control for advanced users.

## Parameter Design

### 1. Core Generation Parameters

#### Simple Usage (Good Defaults)
```bash
# Generates rolling hills with good spawn placement
mapgen --name my_level

# Quick terrain variations
mapgen --name hills --preset hills
mapgen --name mountains --preset mountains
mapgen --name archipelago --preset archipelago
```

#### Advanced Usage (Full Control)
```bash
mapgen --name complex_terrain \
  --terrain-type perlin \
  --seed 12345 \
  --amplitude 15.0 \
  --frequency 0.025 \
  --octaves 4 \
  --water-level 2.0
```

### 2. Complete Parameter Set

```rust
#[derive(Parser)]
#[command(name = "mapgen")]
#[command(about = "Generate procedural terrain maps for Minion ARPG")]
struct Args {
    /// Map name (used for filename if --output not specified)
    #[arg(long, default_value = "generated_map")]
    name: String,

    /// Terrain size in grid cells (WIDTHxHEIGHT)
    #[arg(long, default_value = "64x64")]
    size: String,

    /// Output file path in maps/ directory
    #[arg(long)]
    output: Option<String>,

    /// Player spawn position (X,Y,Z) - Y will be adjusted to terrain height
    #[arg(long, default_value = "0.0,1.0,0.0")]
    player_spawn: String,

    // === TERRAIN GENERATION ===
    
    /// Terrain generation preset (flat, rolling, hills, mountains, archipelago)
    #[arg(long)]
    preset: Option<String>,
    
    /// Terrain type (perlin, ridged, layered) - ignored if preset used
    #[arg(long, default_value = "perlin")]
    terrain_type: String,
    
    /// Random seed for terrain generation
    #[arg(long)]
    seed: Option<u32>,
    
    /// Height variation amplitude in world units
    #[arg(long, default_value = "8.0")]
    amplitude: f32,
    
    /// Base noise frequency (smaller = larger features)
    #[arg(long, default_value = "0.03")]
    frequency: f32,
    
    /// Number of noise octaves (detail layers)
    #[arg(long, default_value = "4")]
    octaves: u32,
    
    /// Noise persistence (detail strength falloff)
    #[arg(long, default_value = "0.5")]
    persistence: f32,
    
    /// Frequency multiplier between octaves
    #[arg(long, default_value = "2.0")]
    lacunarity: f32,
    
    /// Water level height (affects biomes and spawning)
    #[arg(long, default_value = "0.0")]
    water_level: f32,
    
    // === SPAWN ZONE CONFIGURATION ===
    
    /// Number of enemy spawn zones to generate
    #[arg(long, default_value = "5")]
    spawn_zones: u32,
    
    /// Maximum terrain slope for spawn placement (0.0-1.0)
    #[arg(long, default_value = "0.3")]
    max_spawn_slope: f32,
    
    /// Minimum distance between spawn zones
    #[arg(long, default_value = "8.0")]
    min_spawn_distance: f32,
    
    /// Force spawn placement even on suboptimal terrain
    #[arg(long)]
    force_spawns: bool,
    
    // === OUTPUT OPTIONS ===
    
    /// Generate heightmap image for debugging (PNG)
    #[arg(long)]
    debug_heightmap: bool,
    
    /// Generate biome map image for debugging (PNG)
    #[arg(long)]
    debug_biomes: bool,
    
    /// Verbose output showing generation details
    #[arg(long, short)]
    verbose: bool,
}
```

## Preset System

### 1. Terrain Presets
**Purpose:** Provide instant good results for common terrain types

```rust
pub fn create_terrain_preset(name: &str, seed: u32) -> Option<TerrainGenerator> {
    match name {
        "flat" => Some(TerrainGenerator {
            seed,
            algorithm: TerrainAlgorithm::Flat { height: 0.0 },
        }),
        
        "rolling" => Some(TerrainGenerator {
            seed,
            algorithm: TerrainAlgorithm::Perlin {
                amplitude: 5.0,
                frequency: 0.04,
                octaves: 3,
                persistence: 0.6,
                lacunarity: 2.0,
            },
        }),
        
        "hills" => Some(TerrainGenerator {
            seed,
            algorithm: TerrainAlgorithm::Perlin {
                amplitude: 12.0,
                frequency: 0.025,
                octaves: 4,
                persistence: 0.5,
                lacunarity: 2.0,
            },
        }),
        
        "mountains" => Some(TerrainGenerator {
            seed,
            algorithm: TerrainAlgorithm::Ridged {
                amplitude: 25.0,
                frequency: 0.02,
                octaves: 5,
            },
        }),
        
        "archipelago" => Some(TerrainGenerator {
            seed,
            algorithm: TerrainAlgorithm::Layered {
                base: Box::new(TerrainAlgorithm::Perlin {
                    amplitude: 15.0,
                    frequency: 0.015,
                    octaves: 3,
                    persistence: 0.7,
                    lacunarity: 2.0,
                }),
                detail: Box::new(TerrainAlgorithm::Perlin {
                    amplitude: 3.0,
                    frequency: 0.08,
                    octaves: 2,
                    persistence: 0.4,
                    lacunarity: 2.0,
                }),
                detail_weight: 0.3,
            },
        }),
        
        _ => None,
    }
}
```

## User Experience Design

### 1. Progressive Disclosure
**Beginner:** Start with presets and basic parameters
**Intermediate:** Override specific parameters while using presets as base
**Expert:** Full manual control of all parameters

### 2. Helpful Defaults
All defaults chosen to produce usable, interesting terrain:
- **Size:** 64x64 (good performance, sufficient detail)
- **Amplitude:** 8.0 (noticeable hills without extreme variations)
- **Frequency:** 0.03 (balanced feature size)
- **Octaves:** 4 (good detail without over-complexity)

### 3. Parameter Validation & Feedback

```rust
fn validate_args(args: &Args) -> MinionResult<()> {
    // Size validation (existing)
    let (width, height) = parse_size(&args.size)?;
    if width > 512 || height > 512 {
        println!("Warning: Large terrain size may take significant time to generate");
    }
    
    // Amplitude validation
    if args.amplitude < 0.1 {
        return Err(MinionError::InvalidMapData {
            reason: "Amplitude must be at least 0.1".to_string(),
        });
    }
    if args.amplitude > 100.0 {
        println!("Warning: Very high amplitude may create extreme terrain");
    }
    
    // Frequency validation
    if args.frequency <= 0.0 || args.frequency > 1.0 {
        return Err(MinionError::InvalidMapData {
            reason: "Frequency must be between 0.0 and 1.0".to_string(),
        });
    }
    
    // Octaves validation
    if args.octaves == 0 || args.octaves > 8 {
        return Err(MinionError::InvalidMapData {
            reason: "Octaves must be between 1 and 8".to_string(),
        });
    }
    
    // Spawn zone validation
    if args.spawn_zones > 20 {
        println!("Warning: Large number of spawn zones may impact performance");
    }
    
    Ok(())
}
```

## Error Messages & Help

### 1. Context-Aware Error Messages
```rust
// Instead of: "Invalid terrain type"
// Provide: "Invalid terrain type 'foo'. Available types: perlin, ridged, layered"

fn parse_terrain_type(type_str: &str) -> MinionResult<TerrainType> {
    match type_str {
        "perlin" => Ok(TerrainType::Perlin),
        "ridged" => Ok(TerrainType::Ridged),
        "layered" => Ok(TerrainType::Layered),
        _ => Err(MinionError::InvalidMapData {
            reason: format!(
                "Invalid terrain type '{}'. Available types: perlin, ridged, layered", 
                type_str
            ),
        }),
    }
}
```

### 2. Helpful CLI Help Text
```rust
#[command(long_about = r#"
Generate procedural terrain maps for Minion ARPG

QUICK START:
  mapgen --preset hills                    # Generate hilly terrain
  mapgen --name my_level --preset mountains # Custom name with mountain preset

EXAMPLES:
  # Basic terrain with custom parameters
  mapgen --name valley --amplitude 6.0 --frequency 0.04

  # Large detailed terrain
  mapgen --size 128x128 --octaves 5 --preset archipelago

  # Debug terrain generation
  mapgen --preset rolling --debug-heightmap --verbose

PRESETS:
  flat         Flat terrain (testing/arena maps)
  rolling      Gentle rolling hills (good for beginners)
  hills        Moderate hills and valleys
  mountains    Dramatic mountain terrain with ridges
  archipelago  Islands and water features

TERRAIN PARAMETERS:
  --amplitude    Height variation (1.0-50.0, higher = more dramatic)
  --frequency    Feature size (0.01-0.1, lower = larger features)
  --octaves      Detail layers (1-8, higher = more detail)
  --water-level  Sea level height (affects biomes and spawning)
"#)]
```

## Command Examples

### 1. Common Use Cases
```bash
# Quick test map
mapgen --name test --size 32x32 --preset flat

# Balanced gameplay map
mapgen --name level1 --preset rolling --spawn-zones 6

# Dramatic landscape
mapgen --name boss_arena --preset mountains --water-level 5.0

# Custom detailed terrain
mapgen --name custom \
  --amplitude 20.0 \
  --frequency 0.02 \
  --octaves 6 \
  --persistence 0.6

# Debug terrain generation
mapgen --name debug_test \
  --preset hills \
  --debug-heightmap \
  --debug-biomes \
  --verbose
```

### 2. Advanced Workflows
```bash
# Iterative design
mapgen --name iteration1 --seed 12345 --preset hills
# ... test in game ...
mapgen --name iteration2 --seed 12345 --amplitude 10.0 --frequency 0.035
# ... test modifications with same base terrain ...

# Multiple variations from same seed
mapgen --name var_low --seed 999 --amplitude 5.0
mapgen --name var_med --seed 999 --amplitude 10.0  
mapgen --name var_high --seed 999 --amplitude 20.0
```

## Performance Feedback

### 1. Generation Time Estimates
```rust
fn estimate_generation_time(width: u32, height: u32, octaves: u32) -> Duration {
    let cell_count = width * height;
    let complexity_factor = octaves as f32;
    
    // Rough estimates based on benchmarking
    let base_time_ms = match cell_count {
        0..=1024 => 10.0,          // 32x32 or smaller
        1025..=4096 => 50.0,       // 64x64
        4097..=16384 => 200.0,     // 128x128
        16385..=65536 => 800.0,    // 256x256
        _ => 3000.0,               // 512x512+
    };
    
    Duration::from_millis((base_time_ms * complexity_factor) as u64)
}

fn show_generation_estimate(args: &Args) {
    let (width, height) = parse_size(&args.size).unwrap_or((64, 64));
    let estimate = estimate_generation_time(width, height, args.octaves);
    
    if estimate > Duration::from_secs(5) {
        println!("Estimated generation time: {:.1}s", estimate.as_secs_f32());
        println!("Consider using smaller size or fewer octaves for faster generation");
    }
}
```

### 2. Progress Feedback
```rust
fn generate_with_progress(generator: &TerrainGenerator, width: u32, height: u32) {
    println!("Generating {}x{} terrain...", width, height);
    
    let start = std::time::Instant::now();
    
    // Show progress for large terrains
    if width * height > 16384 {
        println!("This may take a moment for large terrain...");
    }
    
    let terrain = generator.generate(width, height, 1.0).unwrap();
    
    let elapsed = start.elapsed();
    println!("Terrain generated in {:.2}s", elapsed.as_secs_f32());
}
```

This CLI design balances simplicity for new users with power for experts, following the project's minimalist principles while providing comprehensive terrain generation capabilities.