# Character Movement Troubleshooting Guide

## Overview

This guide helps debug and fix character movement issues in the Minion game. The movement system uses KinematicCharacterController for click-to-move gameplay with automatic terrain following.

## System Architecture

### Movement Pipeline
1. **Input Detection**: Mouse clicks detected and converted to world coordinates
2. **Target Setting**: Valid ground positions set as movement targets
3. **Movement Calculation**: Direction and distance calculated each frame
4. **Controller Application**: Movement applied to KinematicCharacterController
5. **Transform Update**: Controller output updates character position

### Key Components
- **Player Input System**: Handles mouse clicks and camera raycasting
- **Movement System**: Calculates movement vectors and applies to controller
- **Character Controller**: Rapier's KinematicCharacterController for physics
- **Terrain Following**: Automatic ground snapping and slope climbing

## Common Issues and Solutions

### Issue: Character Not Moving at All

**Symptoms**: Clicking has no effect, character stays in place

**Debug Steps**:
1. Check input detection:
   ```bash
   RUST_LOG=info cargo run 2>&1 | grep "INPUT:"
   ```
   Should show mouse clicks and target calculations

2. Verify movement calculations:
   ```bash
   RUST_LOG=info cargo run 2>&1 | grep "MOVEMENT:"
   ```
   Should show non-zero movement vectors when clicking

3. Check controller application:
   ```bash
   RUST_LOG=info cargo run 2>&1 | grep "CONTROLLER:"
   ```
   Should show controller.translation being set

**Common Causes**:
- Missing components on player entity
- System ordering issues
- Invalid ground raycasting
- Controller configuration problems

### Issue: Character Moves But Ignores Terrain

**Symptoms**: Character floats above or sinks below terrain

**Solutions**:
1. Verify KinematicCharacterController configuration:
   ```rust
   KinematicCharacterController {
       snap_to_ground: Some(CharacterLength::Absolute(0.5)),
       max_slope_climb_angle: 45.0_f32.to_radians(),
       // ... other settings
   }
   ```

2. Check collider setup on terrain meshes

3. Ensure character has appropriate collider (capsule recommended)

### Issue: Character Gets Stuck on Terrain

**Symptoms**: Character stops moving when encountering slopes or obstacles

**Solutions**:
1. Adjust slope climbing settings:
   ```rust
   max_slope_climb_angle: 60.0_f32.to_radians(),  // Allow steeper slopes
   ```

2. Check terrain mesh quality - avoid extreme height changes

3. Verify character collider size isn't too large for terrain features

### Issue: Movement is Jittery or Unstable

**Symptoms**: Character oscillates or moves erratically

**Solutions**:
1. Check movement speed and delta time calculations
2. Verify system execution order
3. Look for multiple systems modifying the same transform
4. Check for NaN values in movement calculations

## Debugging Tools

### Enable Debug Logging
Replace PlayerPlugin with PlayerDebugPlugin in your app setup:
```rust
// In src/plugins/mod.rs
app.add_plugins(PlayerDebugPlugin)  // Instead of PlayerPlugin
```

Configure debug levels:
```rust
app.insert_resource(MovementDebugConfig {
    log_input: true,
    log_calculations: true,
    log_controller: true,
    log_output: true,
    ..default()
})
```

### Debug Commands
```bash
# Run unit tests for movement logic
cargo test game_logic::movement --verbose

# Test minimal movement example
cargo run --example minimal_movement

# Full debug logging
RUST_LOG=info cargo run

# Filtered debug output
RUST_LOG=info cargo run 2>&1 | grep -E "(INPUT|MOVEMENT|CONTROLLER|OUTPUT):"

# Check for errors and warnings
RUST_LOG=info cargo run 2>&1 | grep -E "(WARNING|ERROR):"
```

### Expected Debug Output
With debug logging enabled, you should see:
```
INPUT: cursor(400.0, 300.0) camera(...) 
TARGET: player_pos(...) final_target(...)
MOVEMENT: pos(...) target(...) should_move(true) distance(5.2) movement(...)
CONTROLLER: entity(...) translation(...) slide(true)
OUTPUT: entity(...) effective_translation(...) grounded(true)
```

## Systematic Debugging Protocol

### Step 1: Verify Unit Tests
```bash
cargo test game_logic::movement --verbose
```
All movement calculation tests should pass. If they fail, the issue is in the math logic.

### Step 2: Test Minimal Example
```bash
cargo run --example minimal_movement
```
This tests basic KinematicCharacterController functionality. If this fails, the issue is with the controller setup or Rapier integration.

### Step 3: Check Input System
Enable input logging and verify:
- Mouse clicks are detected
- Camera raycasting works
- Ground intersection finds valid positions
- Target positions are reasonable

### Step 4: Verify Movement Calculations
Enable calculation logging and verify:
- Movement vectors are calculated
- Direction and distance are correct
- Movement is applied when expected

### Step 5: Check Controller Interface
Enable controller logging and verify:
- `controller.translation` is being set
- Values are non-zero when movement intended
- Controller configuration is reasonable

### Step 6: Monitor Controller Output
Enable output logging and verify:
- `KinematicCharacterControllerOutput` exists
- `effective_translation` matches desired movement
- Character reports as grounded
- No collision warnings

### Step 7: Verify Transform Updates
Enable transform logging and verify:
- Transform.translation actually changes
- Position changes match expected movement
- No infinite small movements (jitter)

## Fallback Strategies

If systematic debugging doesn't resolve the issue, consider these alternatives:

### Option 1: Physics-Based Movement (Revert)
Return to previous working system using RigidBody::Dynamic with forces:
```rust
RigidBody::Dynamic,
Velocity::default(),
Damping { linear_damping: 3.0, angular_damping: 8.0 },
ExternalForce::default(),
```

### Option 2: Direct Transform Movement
Bypass character controller and manipulate transforms directly:
```rust
transform.translation += movement;
// Add manual terrain following if needed
```

### Option 3: Alternative Character Controller
Try different character controller libraries:
- bevy-tnua
- bevy_prototype_character_controller

## Technical Configuration

### Required Components for Player
```rust
// Movement components
RigidBody::KinematicPositionBased,
KinematicCharacterController::default(),
KinematicCharacterControllerOutput::default(),

// Collision
Collider::capsule_y(1.0, 0.5),

// Game logic
Player::default(),
Speed(5.0),
```

### System Execution Order
Systems must run in this order:
1. `handle_player_input` - Sets movement targets
2. `move_player` - Calculates and applies movement
3. `update_player_from_controller_output` - Reads controller results

### Common Configuration Issues
- **Missing snap_to_ground**: Character won't follow terrain
- **Wrong RigidBody type**: Must be KinematicPositionBased
- **Incorrect collider**: Use capsule for characters
- **System ordering**: Output reading must happen after controller modification

## Performance Considerations

- Movement calculations are O(1) per character
- KinematicCharacterController has ~0.1ms CPU cost per character
- Terrain following uses efficient collision detection
- Debug logging can impact performance - disable in production

## Files and Locations

### Key Source Files
- `/src/plugins/player.rs` - Main player movement system
- `/src/plugins/player_debug.rs` - Debug-enabled player system
- `/src/game_logic/movement.rs` - Unit testable movement logic
- `/examples/minimal_movement.rs` - Minimal working example

### Testing
- Unit tests: `cargo test game_logic::movement`
- Integration: Run minimal example
- Manual: Enable debug logging and test in game

This troubleshooting guide should help identify and resolve most character movement issues in the Minion game.