# Map Editing Utilities

This directory contains a collection of map editing utilities for manipulating generated maps in the Minion ARPG. These utilities allow fine-tuned control over generated maps and provide a powerful editing pipeline for map customization.

## Available Utilities

### 1. filter_spawns.rs - Spawn Point Filtering
Removes enemy spawn zones that are too close to each other, helping create better distributed spawning.

**Usage:**
```bash
# Remove spawn points within 5.0 units of each other
cargo run --example filter_spawns -- --input hills.bin --output filtered_hills.bin --min-distance 5.0

# Use default min-distance (5.0 units)
cargo run --example filter_spawns -- --input map.bin --output filtered_map.bin

# Dry run to see what would be removed
cargo run --example filter_spawns -- --input map.bin --output filtered_map.bin --dry-run
```

**Features:**
- Priority-based filtering (keeps zones with more max_enemies)
- Dry-run mode for safe previewing
- Verbose output showing which zones are removed
- Configurable minimum distance threshold

### 2. add_objects.rs - Environment Object Placement
Adds environment objects to existing maps with terrain-aware placement and collision avoidance.

**Usage:**
```bash
# Add 50 trees to a map
cargo run --example add_objects -- --input map.bin --output map_with_trees.bin --type tree --count 50

# Add random objects with high density
cargo run --example add_objects -- --input hills.bin --output dense_hills.bin --density 0.8 --count 100

# Add specific object types with custom scale range
cargo run --example add_objects -- --input map.bin --output custom_map.bin --types "tree,rock,bush" --count 75 --scale "0.5,2.0"
```

**Features:**
- Terrain-aware placement (respects slopes and bounds)
- Collision detection with existing objects
- Configurable object types, density, and scale ranges
- Reproducible placement with seed support

### 3. smooth_terrain.rs - Terrain Smoothing
Applies smoothing filters to terrain heightmaps to reduce sharp features and create more natural landscapes.

**Usage:**
```bash
# Apply 3 passes of smoothing with default strength
cargo run --example smooth_terrain -- --input mountains.bin --output smooth_mountains.bin --passes 3

# Light smoothing with custom strength
cargo run --example smooth_terrain -- --input hills.bin --output gentle_hills.bin --passes 1 --strength 0.3

# Strong smoothing with many passes
cargo run --example smooth_terrain -- --input rough_terrain.bin --output flat_terrain.bin --passes 10 --strength 0.8
```

**Features:**
- Multiple smoothing algorithms (box filter, Gaussian)
- Configurable number of passes and strength
- Terrain statistics before and after smoothing
- Preserves terrain bounds and validation

### 4. map_info.rs - Map Analysis and Statistics  
Displays comprehensive information and statistics about map files for analysis and debugging.

**Usage:**
```bash
# Basic map information
cargo run --example map_info -- --input complex_map.bin

# Detailed information with verbose output
cargo run --example map_info -- --input hills.bin --verbose

# Show only terrain information
cargo run --example map_info -- --input map.bin --section terrain

# Export information to JSON format
cargo run --example map_info -- --input map.bin --format json --output map_info.json
```

**Features:**
- Comprehensive terrain analysis (height distribution, statistics)
- Spawn zone breakdown by type and radius
- Environment object analysis by type and position
- JSON export capability
- Sectioned output for focused analysis

### 5. merge_maps.rs - Map Component Merging
Combines terrain, objects, and spawn zones from multiple maps into a single map for modular map composition.

**Usage:**
```bash
# Combine terrain from one map with objects from another
cargo run --example merge_maps -- --terrain hills.bin --objects forest.bin --output combined.bin

# Merge terrain, objects, and spawn zones from different maps
cargo run --example merge_maps -- --terrain mountains.bin --objects trees.bin --spawns arena.bin --output epic_map.bin

# Use a base map and add objects from another
cargo run --example merge_maps -- --base base_map.bin --objects decorations.bin --output enhanced_map.bin
```

**Features:**
- Selective component merging (terrain, objects, spawns, player spawn)
- Object transformation (scaling, offset)
- Replace or merge modes for objects and spawn zones
- Terrain compatibility validation
- Detailed merge statistics

## Utility Pipeline Examples

These utilities are designed to be composable. Here are some common workflows:

### Creating a Custom Map
```bash
# 1. Generate base terrain
cargo run --bin mapgen -- --name custom_level --terrain-type hills --size 128x128 --output base_terrain.bin

# 2. Add environment objects
cargo run --example add_objects -- --input base_terrain.bin --output with_objects.bin --types "tree,rock" --count 200

# 3. Smooth rough areas
cargo run --example smooth_terrain -- --input with_objects.bin --output smoothed.bin --passes 2 --strength 0.4

# 4. Filter overlapping spawn zones
cargo run --example filter_spawns -- --input smoothed.bin --output final_map.bin --min-distance 8.0

# 5. Analyze the result
cargo run --example map_info -- --input final_map.bin --verbose
```

### Combining Multiple Maps
```bash
# Take terrain from mountains, objects from forest, spawns from arena
cargo run --example merge_maps -- --terrain mountains.bin --objects forest_objects.bin --spawns arena_spawns.bin --output epic_battlefield.bin

# Analyze the merged result
cargo run --example map_info -- --input epic_battlefield.bin --verbose
```

### Map Refinement
```bash
# Add more objects to an existing map
cargo run --example add_objects -- --input existing_map.bin --output enhanced_map.bin --type bush --count 50 --density 0.3

# Smooth only problematic terrain areas
cargo run --example smooth_terrain -- --input enhanced_map.bin --output polished_map.bin --passes 1 --strength 0.2

# Check the results
cargo run --example map_info -- --input polished_map.bin --section terrain
```

## Common Options

Most utilities support these common options:
- `--dry-run`: Preview changes without modifying files
- `--verbose` / `-v`: Detailed output and statistics
- `--input`: Input map file (in maps/ directory)
- `--output`: Output map file (in maps/ directory)

## Error Handling

All utilities include comprehensive error handling and validation:
- Input file validation and helpful error messages
- Map data validation using the existing validation system
- Graceful handling of edge cases (empty maps, invalid parameters)
- Type-safe operations with proper bounds checking

## Integration with Existing Systems

These utilities integrate seamlessly with the existing Minion codebase:
- Use existing map data structures and validation
- Follow established CLI patterns from mapgen.rs
- Leverage existing terrain generation and map loading systems
- Compatible with the game's bincode serialization format