# Minion

A Diablo-like action RPG built with Rust and Bevy 0.16, featuring physics-based movement, comprehensive LOD systems, dual UI frameworks, and runtime-configurable gameplay.

## Architecture Overview

### Dual UI Framework
- **egui**: Main menu and settings (immediate mode GUI)
- **Bevy UI**: In-game HUD and tooltips (retained mode)
- Separate camera systems with coordinated render ordering

### Physics-First Movement
- All entities use Rapier 3D `RigidBody::Dynamic` with force-based movement
- High damping prevents physics glitches (3.0 linear, 8.0 angular)
- Capsule colliders with locked rotation axes prevent character tipping
- Enemy flocking via two-pass separation force system

### Comprehensive LOD System
- Global `max_lod_level` setting caps all characters regardless of distance
- Runtime 3D model switching via `SceneRoot` component hot-swapping
- Distance-based quality: Player (always high), Enemies (5m/10m/15m thresholds)
- Preloaded asset handles for instant switching

## Key Features

### Type-Safe Resource Management
```rust
HealthPool, ManaPool, EnergyPool  // Generic pools with phantom types
// Saturating math operations (no negatives)
// Type-specific methods: .is_dead(), .spend(), .take_damage()
```

### Runtime Configuration System
- TOML config at `~/.config/minion/config.toml` with validation
- Graceful degradation: invalid values fall back to defaults individually
- Hot-reload ready for gameplay tuning
- Comprehensive range constraints on all parameters

### Combat & AI Systems
- Deterministic ring-based enemy spawning (counter-based angular distribution)
- Force-based projectile physics rather than transform manipulation
- Area effect system with duration-based cleanup
- Two-pass enemy AI: collect positions → apply separation forces

## Critical Implementation Details

### Physics Configuration
```rust
// Prevent character tipping over
LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z

// Mass properties: Player (1.0) vs Enemy (0.8) density
ColliderMassProperties::Density(1.0)

// High damping prevents oscillation
Damping { linear_damping: 3.0, angular_damping: 8.0 }
```

### LOD Asset Pattern
- **Naming**: `{type}-{level}.glb` (hooded-high.glb, dark-knight-med.glb)
- **Loading**: `{path}#Scene0` for GLTF scene extraction
- **Distance**: 2D calculations only (ignore Y axis) for consistency
- **Switching**: Cheap Handle clones enable instant model swapping

## Development

### Requirements
- Rust 2024 edition
- `sudo apt-get install libasound2-dev` (ALSA for audio)
- Git LFS for asset management (auto-configured)

### Commands
```bash
cargo run              # Launch game (expect 10+ min compile time)
cargo check            # Fast syntax checking
cargo clippy           # Linting
cargo fmt              # Code formatting
```

### Pre-commit Setup
```bash
# Install cargo-llvm-cov for coverage (optional but recommended)
cargo install cargo-llvm-cov

# Install pre-commit hooks
uv tool run pre-commit install

# Run hooks manually on all files
uv tool run pre-commit run --all-files
```

Pre-commit hooks include:
- **rustfmt**: Code formatting
- **clippy**: Linting with all targets and features
- **cargo test**: Run all tests
- **coverage check**: Ensure 85%+ code coverage (requires cargo-llvm-cov)
- **trailing whitespace**: Remove trailing whitespace
- **file fixers**: End-of-file and merge conflict checks

### Asset Management (Git LFS)
- **Automatic**: All 3D models, textures, audio, fonts tracked via LFS
- **Workflow**: Add to `assets/` → `git add` → `git commit` → `git push`
- **New workstation**: `git clone` → `git lfs pull`

## Project Structure
```
src/
├── components/         # ECS components with type-safe resource pools
├── config.rs          # TOML config with validation and graceful fallbacks
├── game_logic/        # Core mechanics (spawning, damage, names)
├── plugins/           # Bevy systems (combat, enemy AI, player, UI)
├── resources/         # Global state and configuration resources
└── main.rs           # Application entry with dual UI setup
```

## Testing & Error Handling
- Comprehensive unit tests for all game logic modules
- Custom `MinionError` type with `thiserror` integration
- Result-based APIs with consistent error propagation
- Graceful degradation in config loading, spawning, and asset loading

---

Built for Rust experts who want to see sophisticated game architecture patterns beyond typical tutorials.
