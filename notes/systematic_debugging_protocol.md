# Systematic Movement Debugging Protocol

## Overview

This protocol provides a step-by-step approach to diagnosing and fixing the character movement system. Each step has clear success/failure criteria and next actions.

## Pre-Debugging Checklist

Before starting, ensure you have:
- [ ] Unit test framework implemented
- [ ] Debug logging enabled
- [ ] Minimal reference example created
- [ ] Current implementation backed up

## Phase 1: Unit Test Validation

### Step 1.1: Test Pure Movement Logic
```bash
cargo test game_logic::movement --verbose
```

**Success Criteria:**
- All movement calculation tests pass
- Ray-to-ground conversion works correctly
- Target validation functions properly

**If Failed:**
- Fix calculation logic before proceeding
- Verify coordinate system assumptions
- Check math operations

### Step 1.2: Test Configuration Generation
Verify MovementConfig values match game settings:
- Player speed
- Stopping distance
- Slowdown distance
- Delta time calculations

## Phase 2: Minimal Example Verification

### Step 2.1: Run Minimal Example
```bash
cargo run --example minimal_movement
```

**Success Criteria:**
- Character spawns correctly
- Click detection works
- Movement calculations execute
- Character actually moves toward targets

**If Failed:**
- Issue is with basic KinematicCharacterController setup
- Check Rapier version compatibility
- Verify system ordering
- Review controller configuration

### Step 2.2: Compare Minimal vs Full Implementation
If minimal example works but full game doesn't:
- Identify configuration differences
- Check for system conflicts
- Look for resource/component differences

## Phase 3: Input System Verification

### Step 3.1: Enable Input Debug Logging
Set `MovementDebugConfig.log_input = true`

**Verify:**
- [ ] Mouse clicks detected
- [ ] Camera ray calculation correct
- [ ] Ground intersection calculation works
- [ ] Target positions are reasonable

**Debug Command:**
```bash
RUST_LOG=info cargo run 2>&1 | grep "INPUT:"
```

### Step 3.2: Validate Target Setting
Set `MovementDebugConfig.log_targets = true`

**Verify:**
- [ ] Player target gets set
- [ ] Target validation passes
- [ ] Target coordinates are world-space correct

## Phase 4: Movement Calculation Verification

### Step 4.1: Enable Movement Debug Logging
Set `MovementDebugConfig.log_calculations = true`

**Verify:**
- [ ] Movement calculations execute
- [ ] Direction vectors are correct
- [ ] Distance calculations match expectations
- [ ] Movement vectors are non-zero when they should be

**Debug Command:**
```bash
RUST_LOG=info cargo run 2>&1 | grep "MOVEMENT:"
```

### Step 4.2: Check System Execution Order
Ensure systems run in correct order:
1. `handle_player_input` (sets target)
2. `move_player` (calculates movement, sets controller)
3. `update_player_from_controller_output` (reads results)

## Phase 5: Controller Interface Verification

### Step 5.1: Enable Controller Debug Logging
Set `MovementDebugConfig.log_controller = true`

**Verify:**
- [ ] Controller.translation is being set
- [ ] Values are non-zero when movement intended
- [ ] Controller configuration is correct

**Debug Command:**
```bash
RUST_LOG=info cargo run 2>&1 | grep "CONTROLLER:"
```

### Step 5.2: Check Controller Output
Set `MovementDebugConfig.log_output = true`

**Critical Checks:**
- [ ] `KinematicCharacterControllerOutput` exists
- [ ] `effective_translation` matches `desired_translation`
- [ ] Character is grounded
- [ ] No warning messages about failed movement

**Debug Command:**
```bash
RUST_LOG=info cargo run 2>&1 | grep "OUTPUT:"
```

## Phase 6: Transform Update Verification

### Step 6.1: Enable Transform Debug Logging
Set `MovementDebugConfig.log_transforms = true`

**Verify:**
- [ ] Transform.translation actually changes
- [ ] Position deltas match expected movement
- [ ] No infinite small movements (jitter)

**Debug Command:**
```bash
RUST_LOG=info cargo run 2>&1 | grep "TRANSFORM:"
```

### Step 6.2: Check Physics Integration
Set `MovementDebugConfig.log_physics = true`

**Verify:**
- [ ] RigidBody type is KinematicPositionBased
- [ ] No conflicting velocity values
- [ ] No physics forces interfering

## Phase 7: System Integration Issues

### Step 7.1: Check for Component Conflicts
Look for systems that might interfere:
- Other movement systems
- Physics systems
- Transform modification systems
- Camera following systems

### Step 7.2: Verify Resource Access
Ensure all systems can access required resources:
- GameConfig
- Time
- Input resources
- Player entities

## Phase 8: Known Issues Investigation

### Step 8.1: Missing Gravity
KinematicCharacterController doesn't automatically handle gravity:

**Test:** Add manual gravity to see if character "falls" to ground
```rust
controller.translation = Some(movement + Vec3::new(0.0, -9.81 * time.delta_secs(), 0.0));
```

### Step 8.2: System Ordering Issues
Based on research, systems reading output must run BEFORE systems modifying controller:

**Current Order:** ✅ Should be correct
```rust
(
    handle_player_input, 
    move_player, 
    update_player_from_controller_output
)
```

### Step 8.3: Snap-to-Ground Configuration
Character might be "floating" and unable to move:

**Test:** Disable snap-to-ground temporarily:
```rust
snap_to_ground: None,
```

### Step 8.4: Offset Issues
Too large offset might prevent movement:

**Test:** Reduce offset:
```rust
offset: CharacterLength::Absolute(0.01),
```

## Debug Commands Summary

```bash
# Run unit tests
cargo test game_logic::movement --verbose

# Run minimal example
cargo run --example minimal_movement

# Full debug logging (very verbose)
RUST_LOG=info cargo run

# Filtered debug logging
RUST_LOG=info cargo run 2>&1 | grep -E "(INPUT|TARGET|MOVEMENT|CONTROLLER|OUTPUT|TRANSFORM):"

# Check specific issues
RUST_LOG=info cargo run 2>&1 | grep "WARNING"
RUST_LOG=info cargo run 2>&1 | grep "ERROR"
```

## Decision Tree

```
1. Do unit tests pass?
   ├─ No → Fix calculation logic, return to step 1
   └─ Yes → Continue to step 2

2. Does minimal example work?
   ├─ No → Basic API issue, check Rapier docs/version
   └─ Yes → Continue to step 3

3. Are inputs being detected?
   ├─ No → Fix input system
   └─ Yes → Continue to step 4

4. Are movement calculations correct?
   ├─ No → Fix calculation or config
   └─ Yes → Continue to step 5

5. Is controller.translation being set?
   ├─ No → Fix movement system
   └─ Yes → Continue to step 6

6. Is controller output showing effective movement?
   ├─ No → Controller configuration issue
   └─ Yes → Continue to step 7

7. Are transforms actually updating?
   ├─ No → System ordering or physics issue
   └─ Yes → Movement should work!
```

## Success Criteria

Movement system is working when:
- [ ] Clicks set valid targets
- [ ] Movement calculations are correct
- [ ] Controller receives movement commands
- [ ] Controller output shows effective movement
- [ ] Character transform updates
- [ ] Character visually moves toward target
- [ ] Character stops at target

## Failure Recovery

If systematic debugging fails to identify the issue:
1. Implement fallback transform-based movement
2. Create GitHub issue with debug logs
3. Seek community help with minimal reproduction case
4. Consider alternative character controller libraries