# Terrain Following Research and Recommendations

## Executive Summary

**Recommended Approach**: **KinematicCharacterController with snap-to-ground** - This provides the best balance of simplicity, performance, and gameplay feel for a Diablo-like ARPG.

## Current System Analysis

### Architecture Overview
- **Movement**: `RigidBody::KinematicPositionBased` with direct transform manipulation
- **Terrain**: Heightmap-based with trimesh collider generation 
- **Heightmap Sampling**: `map.get_height_at_world(x, z)` with bilinear interpolation
- **Current Issues**: Characters maintain static Y altitude, causing floating/sinking

### Key Code Findings
```rust
// Current movement (src/plugins/player.rs:151)
transform.translation += movement;  // Only X/Z movement, no Y adjustment

// Terrain sampling available (src/map/mod.rs:143)
pub fn get_height_at_world(&self, world_x: f32, world_z: f32) -> Option<f32>

// Spawn positioning already uses terrain height (src/plugins/player.rs:53-55)
let terrain_height_at_spawn = map.get_height_at_world(map.player_spawn.x, map.player_spawn.z);
let final_y = terrain_height_at_spawn.unwrap_or(map.player_spawn.y) + 0.5;
```

## Approach Evaluation

### 1. Gravity + Physics Collision (Not Recommended)
```rust
// Would require switching to Dynamic bodies
RigidBody::Dynamic,
GravityScale(1.0),
```

**Pros:**
- Realistic physics behavior
- Handles complex terrain automatically
- Natural falling/jumping mechanics

**Cons:**
- **Major architectural change** from kinematic to dynamic movement
- **Physics complexity** - requires retuning damping, mass, forces
- **Potential glitches** - characters could get stuck, bounce, or behave unpredictably
- **Performance overhead** - continuous physics simulation
- **Control complexity** - harder to achieve precise ARPG movement feel

**Verdict**: ❌ Too disruptive for current kinematic architecture

### 2. Ground Clamping/Heightmap Sampling (Moderate Recommendation)
```rust
// Add to movement systems
if let Some(terrain_height) = map.get_height_at_world(transform.translation.x, transform.translation.z) {
    transform.translation.y = terrain_height + character_height_offset;
}
```

**Pros:**
- **Simple implementation** - minimal code changes
- **Predictable behavior** - no physics surprises
- **Performance efficient** - single heightmap lookup per character
- **Maintains current architecture** - works with existing kinematic movement

**Cons:**
- **No physics interactions** - can't fall into holes or off cliffs
- **Instant teleportation** - characters snap to terrain height immediately
- **Manual implementation** - need to handle edge cases, interpolation
- **Limited gameplay** - no jumping, falling, knockback effects

**Implementation Complexity**: Low (1-2 days)

### 3. Hybrid Approach (Moderate Recommendation)
```rust
enum MovementMode {
    GroundFollowing,  // Use heightmap clamping
    Physics,          // Use gravity + collision
}
```

**Pros:**
- **Best of both worlds** - smooth terrain following + physics when needed
- **Flexible gameplay** - supports both normal movement and special effects
- **Incremental implementation** - can start with ground clamping, add physics later

**Cons:**
- **Implementation complexity** - managing mode transitions
- **Potential inconsistencies** - different behaviors in different modes
- **More code to maintain** - two movement systems to debug

**Implementation Complexity**: Medium (3-5 days)

### 4. Raycast-Based Terrain Following (Lower Priority)
```rust
// Cast ray downward from character
if let Some((_, hit)) = rapier_context.cast_ray(
    character_pos + Vec3::Y * 2.0,  // Start above character
    Vec3::NEG_Y,                    // Cast downward
    10.0,                          // Max distance
    true,                          // Solid objects only
    QueryFilter::default()
) {
    transform.translation.y = hit.point.y + character_height_offset;
}
```

**Pros:**
- **Works with any collision geometry** - not limited to heightmaps
- **Accurate collision detection** - uses same physics system as gameplay
- **Handles complex scenarios** - overhangs, bridges, multi-level terrain

**Cons:**
- **Performance overhead** - raycasts every frame for every character
- **Implementation complexity** - filtering, edge cases, ray management
- **May conflict with colliders** - could interfere with existing physics

**Implementation Complexity**: Medium (2-4 days)

### 5. KinematicCharacterController (RECOMMENDED)
```rust
// Replace current movement with Rapier's character controller
RigidBody::KinematicPositionBased,
KinematicCharacterController {
    snap_to_ground: Some(CharacterLength::Absolute(0.5)),
    max_slope_climb_angle: 45.0_f32.to_radians(),
    min_slope_slide_angle: 30.0_f32.to_radians(),
    ..default()
},
```

**Pros:**
- **Built for this purpose** - designed specifically for character terrain following
- **Production tested** - used in many shipped games
- **Maintains kinematic movement** - compatible with current architecture
- **Rich feature set** - slope climbing, ground snapping, stair stepping
- **Performance optimized** - efficient collision detection
- **Handles edge cases** - moving platforms, steep slopes, obstacles

**Cons:**
- **Learning curve** - need to understand controller configuration
- **Migration effort** - replace current movement system
- **Rapier dependency** - tied to specific physics engine version

**Implementation Complexity**: Medium (2-3 days for basic integration)

## Technical Recommendation

### Primary Approach: KinematicCharacterController

**Why this is optimal for Minion:**

1. **Perfect fit for ARPG gameplay** - Designed for precisely this use case
2. **Minimal architectural disruption** - Works with existing kinematic movement
3. **Future-proof** - Handles complex scenarios as game grows
4. **Well-maintained** - Active development in Rapier ecosystem
5. **Performance optimized** - Efficient collision detection

### Implementation Strategy

#### Phase 1: Basic Integration (Priority: High)
```rust
// 1. Add character controller component to player/enemies
KinematicCharacterController {
    snap_to_ground: Some(CharacterLength::Absolute(0.5)),
    max_slope_climb_angle: 45.0_f32.to_radians(),
    min_slope_slide_angle: 30.0_f32.to_radians(),
    ..default()
}

// 2. Replace direct transform manipulation with controller movement
// Instead of: transform.translation += movement;
// Use: character_controller.translation = Some(movement);
```

#### Phase 2: Movement Integration (Priority: High)
- Replace player movement system to use `KinematicCharacterController.translation`
- Replace enemy AI movement to use character controller
- Remove manual Y positioning at spawn (controller handles it)

#### Phase 3: Configuration Tuning (Priority: Medium)
- Adjust `snap_to_ground` distance for optimal feel
- Configure slope angles for gameplay balance
- Test with various terrain types

#### Phase 4: Advanced Features (Priority: Low)
- Add jump mechanics using `KinematicCharacterController.translation.y`
- Implement knockback effects through controller
- Add moving platform support

### Alternative: Heightmap Clamping (Fallback Option)

If KinematicCharacterController proves problematic:

```rust
fn terrain_following_system(
    mut query: Query<&mut Transform, With<TerrainFollower>>,
    map: Res<MapDefinition>,
) {
    for mut transform in query.iter_mut() {
        if let Some(terrain_height) = map.get_height_at_world(
            transform.translation.x, 
            transform.translation.z
        ) {
            // Smooth interpolation instead of instant snap
            let target_y = terrain_height + 0.5; // Character height offset
            let current_y = transform.translation.y;
            let lerp_speed = 10.0; // Adjust for smoothness
            transform.translation.y = current_y.lerp(target_y, lerp_speed * time.delta_secs());
        }
    }
}
```

## Performance Considerations

### KinematicCharacterController
- **CPU Cost**: ~0.1ms per character on modern hardware
- **Memory**: ~200 bytes per character controller
- **Scalability**: Excellent - designed for many characters

### Heightmap Clamping
- **CPU Cost**: ~0.01ms per character (single heightmap lookup)
- **Memory**: ~0 bytes additional
- **Scalability**: Perfect - O(1) per character

### Recommendation
For current game scale (player + 10-20 enemies), either approach performs excellently. KinematicCharacterController is still recommended for its features and future flexibility.

## Integration with Existing Systems

### Compatibility Assessment
- ✅ **Kinematic movement**: KinematicCharacterController maintains kinematic approach
- ✅ **LOD system**: No impact on existing LOD implementation
- ✅ **Combat system**: Character controller output provides collision info
- ✅ **Spawning system**: Remove manual Y positioning, let controller handle it
- ⚠️ **Camera system**: May need adjustment if character Y position changes more dynamically

### Required Changes Summary
1. **Add KinematicCharacterController component** to player and enemy spawning
2. **Replace movement logic** in player and enemy systems
3. **Remove manual Y positioning** from spawn functions
4. **Add configuration tuning** systems for controller parameters

## Conclusion

**KinematicCharacterController** provides the optimal solution for terrain following in Minion. It offers:

- **Production-ready implementation** for character terrain following
- **Minimal disruption** to existing kinematic architecture  
- **Rich feature set** that handles edge cases automatically
- **Performance optimization** suitable for ARPG gameplay
- **Future flexibility** for advanced movement mechanics

**Estimated implementation time**: 2-3 days for basic integration, 1 additional day for tuning and polish.

The existing heightmap sampling infrastructure remains valuable and can be used alongside the character controller for gameplay logic (spawn point validation, AI pathfinding, etc.).