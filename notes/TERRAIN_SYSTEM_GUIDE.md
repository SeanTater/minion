# Terrain System Developer Guide

## Overview

The Minion terrain system consists of multiple integrated components working together to generate, render, and interact with procedural terrain. This guide covers everything a new developer needs to know about the terrain pipeline.

## System Architecture

### Core Components

1. **Terrain Generation** (`src/terrain/`)
   - **TerrainData**: Height grid storage and sampling
   - **TerrainGenerator**: Procedural generation using noise functions
   - **BiomeMap**: Terrain classification for gameplay logic

2. **Mesh Generation** (`src/terrain/`)
   - Height-to-mesh conversion with LOD support
   - Physics collider generation from heightmaps
   - Texture coordinate generation

3. **Map Tools** (`src/bin/mapgen.rs`)
   - CLI tool for generating map files
   - Preset system for common terrain types
   - Debug visualization tools

### Key Data Flow

```
MapGen CLI → TerrainGenerator → TerrainData → Mesh Generation → Game Loading
```

## Terrain Generation

### Using the MapGen CLI

Basic usage with presets:
```bash
# Generate common terrain types
mapgen --preset rolling --name my_level
mapgen --preset mountains --name epic_boss_arena
mapgen --preset archipelago --name water_world
```

Advanced usage with custom parameters:
```bash
mapgen --name custom_terrain \
  --amplitude 15.0 \
  --frequency 0.025 \
  --octaves 4 \
  --water-level 2.0 \
  --spawn-zones 6
```

Debug terrain generation:
```bash
mapgen --preset hills --debug-heightmap --debug-biomes --verbose
```

### Terrain Algorithm Types

**Perlin Noise** (default):
- Good for rolling hills and natural-looking terrain
- Parameters: amplitude, frequency, octaves, persistence, lacunarity

**Ridged Noise**:
- Creates mountain ridges and dramatic terrain features
- Parameters: amplitude, frequency, octaves

**Layered Noise**:
- Combines multiple noise types for complex terrain
- Allows base terrain + detail layers

**Flat Terrain**:
- For testing and arena-style maps
- Single height value across entire map

### Parameter Guide

- **Amplitude**: Height variation (1.0-50.0, higher = more dramatic)
- **Frequency**: Feature size (0.01-0.1, lower = larger features)
- **Octaves**: Detail layers (1-8, higher = more detail)
- **Water Level**: Sea level height (affects biomes and spawning)

## Spawn Zone Intelligence

The terrain system includes intelligent spawn placement that avoids steep slopes and water:

### Spawn Suitability Factors
- **Slope Analysis**: Avoids terrain steeper than 0.3 radians
- **Biome Classification**: Excludes water and extreme elevations
- **Accessibility**: Ensures reachable from player spawn
- **Distance Constraints**: Maintains minimum separation between zones

### Configuration Options
```bash
mapgen --spawn-zones 8 \        # Number of enemy spawn zones
       --max-spawn-slope 0.3 \  # Maximum terrain slope
       --min-spawn-distance 8.0 # Minimum distance between zones
```

## Terrain Following System

Characters automatically follow terrain height using KinematicCharacterController:

### Key Features
- **Automatic Slope Climbing**: Characters climb gentle slopes naturally
- **Ground Snapping**: Characters stick to terrain surface
- **Collision Integration**: Works with existing physics system

### Configuration
```rust
KinematicCharacterController {
    snap_to_ground: Some(CharacterLength::Absolute(0.5)),
    max_slope_climb_angle: 45.0_f32.to_radians(),
    min_slope_slide_angle: 30.0_f32.to_radians(),
    ..default()
}
```

## Performance Targets

- **64x64 terrain**: < 100ms generation
- **256x256 terrain**: < 2s generation
- **512x512 terrain**: < 10s generation (warning shown)

Large terrain sizes show progress feedback and time estimates.

## Integration with Game Systems

### Physics System
- Terrain generates trimesh colliders for physics interactions
- Characters use KinematicCharacterController for terrain following
- Projectiles and area effects interact with terrain colliders

### Rendering System
- Terrain meshes integrate with LOD system
- Supports texture coordinate generation for future texturing
- Compatible with existing material and lighting systems

### Gameplay Systems
- Spawn zones placed based on terrain suitability
- Biome system affects enemy types and behavior
- Water level affects movement and visual effects

## Common Workflows

### Creating a New Map
1. Start with a preset that matches your vision:
   ```bash
   mapgen --preset hills --name test_level
   ```

2. Test in game to see how it feels

3. Adjust parameters incrementally:
   ```bash
   mapgen --name test_level_v2 --seed 12345 --amplitude 12.0 --frequency 0.03
   ```

4. Use same seed to maintain base terrain while tweaking parameters

### Debugging Generation Issues
1. Enable debug output:
   ```bash
   mapgen --preset rolling --debug-heightmap --verbose
   ```

2. Check generated PNG files in assets/maps/ directory

3. Verify spawn zone placement makes sense

4. Test loading in game to ensure physics work correctly

### Performance Optimization
- Use smaller map sizes (64x64) during development
- Reduce octaves if generation is slow
- Consider presets before custom parameters
- Use debug tools to verify terrain before expensive game testing

## Technical Details

### Noise Library Choice
- **Primary**: `noise-functions` for performance and simplicity
- **Fallback**: `noise-rs` if more features needed
- Uses f32 precision matching TerrainData structure

### Memory Usage
- TerrainData stores height values in linear Vec\<f32\>
- Biome maps generated only when spawn placement needed
- Mesh generation uses temporary allocations

### Error Handling
- Graceful degradation when ideal spawn placement impossible
- Validation prevents extreme terrain that breaks physics
- Clear error messages guide users to valid parameter ranges

## File Locations

### Key Source Files
- `/src/terrain/` - Core terrain system
- `/src/bin/mapgen.rs` - Map generation CLI
- `/src/map/` - Map loading and data structures

### Generated Files
- `/assets/maps/*.bin` - Map definition files
- `/assets/maps/*_heightmap.png` - Debug heightmap images (when --debug-heightmap used)
- `/assets/maps/*_biomes.png` - Debug biome images (when --debug-biomes used)

This system provides powerful terrain generation while maintaining the project's focus on simplicity and performance.
