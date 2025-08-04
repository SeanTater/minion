# Architecture Decisions and Technical Rationale

## Overview

This document captures key architectural decisions made during development, their rationale, and implications for future development.

## Movement System Architecture

### Decision: KinematicCharacterController over Physics-Based Movement

**Rationale**:
- **Predictable Behavior**: Kinematic movement provides consistent, predictable character movement essential for ARPG gameplay
- **Terrain Integration**: Built-in ground snapping and slope climbing handle terrain following automatically
- **Performance**: More efficient than full physics simulation for character movement
- **Control Precision**: Easier to achieve precise click-to-move behavior

**Implementation Details**:
```rust
RigidBody::KinematicPositionBased,
KinematicCharacterController {
    snap_to_ground: Some(CharacterLength::Absolute(0.5)),
    max_slope_climb_angle: 45.0_f32.to_radians(),
    min_slope_slide_angle: 30.0_f32.to_radians(),
    ..default()
}
```

**Trade-offs**:
- ✅ Predictable movement behavior
- ✅ Automatic terrain following
- ✅ Good performance characteristics
- ❌ Less realistic physics interactions
- ❌ Requires manual implementation of knockback effects

**Alternative Considered**: Physics-based movement with `RigidBody::Dynamic`
- Rejected due to unpredictability and tuning complexity
- Fallback option if kinematic approach fails

## Terrain Generation Architecture

### Decision: noise-functions Library

**Rationale**:
- **Performance**: Static permutation tables provide better cache locality
- **Precision Match**: f32 precision matches existing TerrainData structure
- **Simplicity**: Functional API aligns with project minimalist principles
- **Memory Efficiency**: Zero-allocation approach optimized for mapgen tool

**Implementation**:
```rust
use noise_functions::*;

let height = fbm_2d(x, y, seed, octaves);
let ridged = ridged_2d(x, y, seed, octaves);
```

**Trade-offs**:
- ✅ Optimal performance for use case
- ✅ Simple, functional API
- ✅ Direct f32 compatibility
- ❌ Smaller ecosystem than noise-rs
- ❌ Fewer advanced noise types

**Fallback**: noise-rs library provides more features if limitations discovered

### Decision: Heightmap-Based Terrain

**Rationale**:
- **Simplicity**: 2D height grids are easy to generate, store, and sample
- **Performance**: Efficient collision detection and rendering
- **Tool Compatibility**: Easy to visualize and debug with standard image formats
- **Memory Efficiency**: Compact representation for large terrain areas

**Implementation**:
```rust
pub struct TerrainData {
    pub heights: Vec<f32>,  // Linear array of height values
    pub width: u32,
    pub height: u32,
    pub scale: f32,
}
```

**Trade-offs**:
- ✅ Simple and efficient
- ✅ Easy to generate and manipulate
- ✅ Good tool support
- ❌ Cannot represent overhangs or caves
- ❌ Limited to single height per x,z coordinate

## Physics Integration

### Decision: Rapier Physics Engine

**Rationale**:
- **Bevy Integration**: Official Bevy physics integration
- **Feature Completeness**: Supports both rigid body and character controller needs
- **Performance**: Optimized for real-time games
- **Rust Native**: No FFI overhead, excellent Rust integration

**Implementation**:
- Player/enemies use KinematicCharacterController
- Terrain uses trimesh colliders generated from heightmaps
- Projectiles and items use RigidBody::Dynamic

**Trade-offs**:
- ✅ Excellent Bevy integration
- ✅ Good performance characteristics
- ✅ Comprehensive feature set
- ❌ Large dependency footprint
- ❌ Learning curve for advanced features

## Rendering and LOD System

### Decision: Distance-Based LOD with Preloaded Models

**Rationale**:
- **Performance**: Reduces rendering load for distant characters
- **Flexibility**: Runtime switching without loading delays
- **Quality Control**: Manual LOD creation ensures good quality at all distances
- **Player Experience**: Player always uses high LOD for third-person view

**Implementation**:
```rust
// All LOD levels preloaded at spawn
SceneRoot(asset_server.load("hooded-high.glb#Scene0"))

// Runtime switching based on distance
if distance > far_threshold {
    scene_root.0 = low_lod_handle.clone();
}
```

**Trade-offs**:
- ✅ No runtime loading hitches
- ✅ Predictable performance
- ✅ High visual quality
- ❌ Higher memory usage (all LODs loaded)
- ❌ Manual LOD model creation required

## Configuration System

### Decision: TOML with Runtime Validation

**Rationale**:
- **Human Readable**: TOML is easy to read and edit
- **Type Safety**: Serde provides compile-time validation
- **Runtime Flexibility**: validator crate enables runtime constraint checking
- **Graceful Degradation**: Invalid values fall back to defaults individually

**Implementation**:
```rust
#[derive(Deserialize, Validate)]
pub struct GameConfig {
    #[validate(range(min = 1.0, max = 20.0))]
    player_speed: f32,

    #[validate(range(min = 1, max = 100))]
    max_enemies: u32,
}
```

**Trade-offs**:
- ✅ Easy to read and modify
- ✅ Type-safe with validation
- ✅ Graceful error handling
- ❌ TOML limitations for complex data structures
- ❌ Runtime validation overhead (minimal)

## Error Handling Strategy

### Decision: Custom Error Types with thiserror

**Rationale**:
- **Context Preservation**: Custom errors maintain context through call stack
- **User-Friendly Messages**: thiserror enables clear error descriptions
- **Composability**: Easy to combine different error types
- **Debugging Support**: Good integration with debugging tools

**Implementation**:
```rust
#[derive(Error, Debug)]
pub enum MinionError {
    #[error("Invalid map data: {reason}")]
    InvalidMapData { reason: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type MinionResult<T> = Result<T, MinionError>;
```

**Trade-offs**:
- ✅ Clear error messages
- ✅ Good debugging experience
- ✅ Composable error handling
- ❌ Slight performance overhead
- ❌ Additional complexity for simple cases

## UI System Architecture

### Decision: Dual UI Framework (egui + Bevy UI)

**Rationale**:
- **Tool Integration**: egui excellent for immediate-mode tools and debugging
- **Game Integration**: Bevy UI better for game HUD and world-space elements
- **Development Velocity**: Use right tool for each job
- **Camera Separation**: Different render layers prevent conflicts

**Implementation**:
- **egui**: Main menu, settings, debug tools
- **Bevy UI**: In-game HUD, tooltips, health bars
- **Separate cameras**: Different render orders and layers

**Trade-offs**:
- ✅ Optimal tool for each use case
- ✅ Fast development of debug tools
- ✅ High-quality game UI
- ❌ Two UI systems to maintain
- ❌ Potential styling inconsistencies

## Asset Pipeline

### Decision: Git LFS for Binary Assets

**Rationale**:
- **Repository Size**: Keeps Git repository fast and lightweight
- **Version Control**: Full versioning for binary assets
- **Bandwidth Efficiency**: Only download assets when needed
- **Team Collaboration**: Shared asset storage without repository bloat

**Implementation**:
- All `.glb`, `.png`, `.wav`, etc. automatically tracked by LFS
- Standard Git workflow for asset changes
- `git lfs pull` for new workstation setup

**Trade-offs**:
- ✅ Fast repository operations
- ✅ Full asset versioning
- ✅ Bandwidth efficiency
- ❌ Git LFS dependency
- ❌ Additional setup complexity for new developers

## Type Safety Strategy

### Decision: Phantom Types for Resource Pools

**Rationale**:
- **Type Safety**: Prevents mixing health and mana operations
- **Zero Cost**: Phantom types have no runtime overhead
- **API Clarity**: Clear distinction between different resource types
- **Maintainability**: Compile-time errors catch logical mistakes

**Implementation**:
```rust
pub struct HealthPool(ResourcePool<HealthType>);
pub struct ManaPool(ResourcePool<ManaType>);

// Compile error if you try to transfer health to mana
health_pool.transfer_to(&mut mana_pool); // ❌ Won't compile
```

**Trade-offs**:
- ✅ Strong compile-time guarantees
- ✅ Zero runtime cost
- ✅ Self-documenting code
- ❌ Slightly more verbose API
- ❌ Learning curve for phantom type patterns

## Future Architecture Considerations

### Extensibility Points

1. **Terrain System**: Designed to support additional terrain types and algorithms
2. **Movement System**: Can be extended with additional character controllers
3. **LOD System**: Ready for automatic LOD generation tools
4. **Configuration**: Easy to add new validated configuration options

### Performance Scaling

1. **Terrain Generation**: Supports tiled generation for larger worlds
2. **Entity Management**: LOD system provides foundation for large entity counts
3. **Physics Optimization**: Spatial partitioning ready for dense environments

### Maintenance Strategy

1. **Minimize Dependencies**: Choose well-maintained, stable dependencies
2. **Clear Abstractions**: Hide complexity behind simple interfaces
3. **Comprehensive Testing**: Unit tests for complex logic, integration tests for workflows
4. **Documentation**: Architecture decisions documented for future maintainers

These architectural decisions provide a solid foundation for the game while maintaining flexibility for future development needs.
