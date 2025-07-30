# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Getting Started
When starting work on this codebase, run `./find-definitions.sh` to get an overview of all code structures and their relationships.

## Project Overview
Minion is a Diablo-like action RPG built with Rust and Bevy 0.16. Modular plugin-based architecture with ECS systems, strong typing, and comprehensive error handling.

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

1. **handle_mouse_clicks**: Mouse raycast to ground plane, sets Player.move_target
2. **move_player**: Interpolates Player position toward target, rotates to face direction  
3. **follow_camera**: Updates camera position with fixed offset from Player
4. **setup_scene**: Initializes 3D scene (ground, lighting, camera) 
5. **spawn_player**: Creates Player entity with red capsule mesh

## Key Components
```rust
Player {
    move_target: Option<Vec3>,  // Click target position
    speed: f32,                 // Movement speed (5.0)
}

CameraFollow {
    offset: Vec3,              // Fixed camera offset (10,15,10)
}

ObjectPool<T>                  // Generic pooling system (unused, ready for projectiles)
```

## System Interactions
- **Input Flow**: Mouse click → raycast to ground → Player.move_target set
- **Movement Flow**: Player.move_target → smooth interpolation → Transform update
- **Camera Flow**: Player Transform → camera position update with offset
- **Rendering**: Isometric camera at (10,15,10) → PBR scene with shadows

## Critical Implementation Details
- **Raycasting**: Viewport to world coordinates, intersects with ground plane (y=0)
- **Movement**: Uses `distance_to()` with 0.1 unit threshold for arrival
- **Camera**: Always `look_at()` player position, maintains fixed isometric offset
- **Rendering**: 20x20 ground plane, directional light with shadows, 300 brightness ambient

## Extension Points
The single-file architecture is ready for modularization. Natural split points:
- Player movement system → `player.rs`
- Camera controls → `camera.rs` 
- Input handling → `input.rs`
- Object pooling expansion for bullets/effects
- Game state management for menus/levels

## Interaction Guidelines
- Don't run the game yourself (e.g. with `cargo run`) - graphics don't work in your terminal. Instead, ask the user to run it