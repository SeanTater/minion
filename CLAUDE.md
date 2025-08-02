# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Getting Started
When starting work on this codebase, run `./find-definitions.sh` to get an overview of all code structures and their relationships.

## Project Overview
Minion is a Diablo-like ARPG built with Rust/Bevy 0.16, featuring physics-based movement, comprehensive LOD systems, dual UI frameworks (egui + Bevy UI), and runtime-configurable gameplay with validation.

## Development Commands
```bash
# Run the game
cargo run

# Build and check code  
cargo build --release
cargo check
cargo fmt
cargo clippy

# System requirement: sudo apt-get install libasound2-dev
```

## Architecture Highlights

### Dual UI System
- **egui**: Main menu and settings (immediate mode)
- **Bevy UI**: In-game HUD and tooltips (retained mode)
- Separate camera systems with render order coordination

### Physics-First Movement
- All entities use `RigidBody::Dynamic` with force-based movement
- High damping values (3.0 linear, 8.0 angular) prevent physics glitches
- Capsule colliders with locked rotation axes prevent character tipping
- Enemy flocking uses separation forces in two-pass system

### Comprehensive LOD System
- Global `max_lod_level` setting caps all characters regardless of distance
- Runtime model switching via `SceneRoot` component changes
- Player always uses close-distance LOD for third-person view
- All LOD levels preloaded at spawn, switched dynamically

## Key Systems & Implementation Details

### Type-Safe Resource Pools
```rust
// Generic pools with phantom types
HealthPool, ManaPool, EnergyPool
// Saturating math - all operations clamp to valid ranges
// Type-specific methods: .is_dead(), .spend(), .take_damage()
```

### Configuration System
- Runtime validation with `validator` crate and range constraints
- TOML persistence at `~/.config/minion/config.toml`
- Graceful degradation - invalid values fall back to defaults individually
- Hot-reload ready via resource updates

### Combat & Spawning
- Deterministic ring-based enemy spawning (counter * prime multiplier)
- Area effect system with duration-based despawning
- Bullet pooling infrastructure ready but unused
- Force-based projectiles rather than transform manipulation

### Critical Physics Gotchas
```rust
// Prevent character tipping
LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z
// Different masses: Player (1.0) vs Enemy (0.8) density
// High damping prevents oscillation
Damping { linear_damping: 3.0, angular_damping: 8.0 }
```

### LOD Performance Pattern
- 2D distance calculations only (ignore Y axis) for consistency
- Cheap Handle clones for model switching
- Asset naming: `{type}-{level}.glb` (hooded-high.glb, dark-knight-med.glb)
- GLTF scenes: `{path}#Scene0` format

## Error Handling & Testing
- Custom `MinionError` with `thiserror` integration
- Comprehensive unit tests for all game logic modules
- Graceful degradation in config loading, spawn positioning, asset loading
- Result-based APIs with consistent error propagation

## Interaction Guidelines
- Don't run the game yourself (e.g. with `cargo run`) - graphics don't work in your terminal
- Expect 10+ minute compilation times on slow systems - use long timeouts
- Long asset uploads with Git LFS are normal

## Git LFS Asset Management
**Workflow**: Add assets to `assets/` → `git add` (auto-LFS) → `git commit` → `git push`
**Tracked**: `.glb`, `.gltf`, `.fbx`, `.obj`, `.png`, `.jpg`, `.wav`, `.ogg`, `.ttf`, `.otf`
**New workstation setup**: `git clone` → `git lfs pull`