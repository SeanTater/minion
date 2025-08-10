# Development Workflows and Best Practices

## Overview

This guide covers common development workflows, debugging strategies, and best practices for working with the Minion codebase.

## Core Development Commands

### Building and Running
```bash
# Run the game
cargo run

# Build for release
cargo build --release

# Check code without building
cargo check

# Format code
cargo fmt

# Lint code
cargo clippy
```

### System Requirements
- Install audio dependencies: `sudo apt-get install libasound2-dev`
- Expect 10+ minute compilation times on slow systems
- Use long timeouts for asset loading

## Map Generation Workflow

### Creating New Maps

1. **Start with presets** for rapid iteration:
   ```bash
   mapgen --preset rolling --name test_map
   ```

2. **Test in game** to evaluate feel and gameplay

3. **Iterate with parameters**:
   ```bash
   # Use same seed to maintain base terrain
   mapgen --name test_map_v2 --seed 12345 --amplitude 10.0
   mapgen --name test_map_v3 --seed 12345 --amplitude 15.0 --frequency 0.035
   ```

4. **Use debug tools** when things don't look right:
   ```bash
   mapgen --preset hills --debug-heightmap --debug-biomes --verbose
   ```

5. **Validate in game** - check spawn zones, physics, performance

### Map Iteration Strategy
- Keep map sizes small (64x64) during development
- Use presets as starting points rather than building from scratch
- Generate debug images to visualize without game loading
- Use consistent seeds for comparative testing

## Debugging Movement Issues

### Quick Diagnostic
```bash
# Test basic functionality
cargo test game_logic::movement --verbose

# Test minimal controller setup
cargo run --example minimal_movement

# Check for obvious errors
RUST_LOG=info cargo run 2>&1 | grep -E "(WARNING|ERROR):"
```

### Deep Debugging
1. **Enable debug player system** in your plugin configuration
2. **Use filtered logging** to focus on specific issues:
   ```bash
   RUST_LOG=info cargo run 2>&1 | grep "INPUT:"     # Check input detection
   RUST_LOG=info cargo run 2>&1 | grep "MOVEMENT:"  # Check calculations
   RUST_LOG=info cargo run 2>&1 | grep "CONTROLLER:" # Check controller state
   ```

3. **Follow systematic protocol** from MOVEMENT_TROUBLESHOOTING.md

### When to Use Fallback Strategies
- If systematic debugging doesn't identify root cause within 2-3 hours
- If KinematicCharacterController proves incompatible with game architecture
- If movement becomes blocking for other development work

## Testing Strategy

### Unit Testing
Focus on complex logic and bug-prone areas:
```bash
# Test specific modules
cargo test game_logic::movement --verbose
cargo test terrain::generator --verbose

# Run all tests
cargo test
```

### Integration Testing
- Test full pipelines (mapgen → game loading)
- Verify system interactions work correctly
- Use minimal examples to isolate issues

### Manual Testing
- Always test major changes in game
- Pay attention to edge cases (steep terrain, water, boundaries)
- Verify performance doesn't regress

## Performance Considerations

### Compilation
- First build can take 10+ minutes on slow systems
- Use `cargo check` for faster syntax validation
- Consider `cargo build --release` for performance testing

### Runtime
- Terrain generation: 64x64 < 100ms, 256x256 < 2s
- Movement systems: ~0.1ms per character
- Asset loading: Git LFS assets may take time to download

### Optimization Workflow
1. **Profile before optimizing** - measure actual bottlenecks
2. **Focus on hot paths** - movement and rendering systems
3. **Test with realistic data** - multiple enemies, complex terrain
4. **Validate optimizations** - ensure behavior unchanged

## Asset Management

### Git LFS Workflow
All game assets use Git LFS for efficient storage:

```bash
# Normal workflow (LFS automatic)
git add assets/models/new_character.glb
git commit -m "Add new character model"
git push

# New workstation setup
git clone <repo>
git lfs pull  # Download LFS assets
```

### Asset Requirements
- Models: `.glb`, `.gltf`, `.fbx`, `.obj`
- Textures: `.png`, `.jpg`
- Audio: `.wav`, `.ogg`
- Fonts: `.ttf`, `.otf`

## Code Style and Patterns

### Error Handling
- Use `MinionResult<T>` for fallible operations
- Prefer `?` operator for error propagation
- Provide helpful error messages with context

### Configuration
- Use validator crate for runtime validation
- Implement graceful degradation for invalid configs
- Provide sensible defaults

### System Design
- Keep systems focused and single-purpose
- Use resources for shared state
- Prefer composition over inheritance

## Architecture Principles

### Minimalism
- Prefer editing existing files over creating new ones
- Consolidate similar functionality
- Remove code that doesn't add value

### Performance First
- Consider performance implications of design choices
- Use appropriate data structures for access patterns
- Profile before optimizing, but design with performance in mind

### Maintainability
- Write code that's easy to understand and modify
- Use clear naming and documentation
- Design for future extensibility without over-engineering

## Common Pitfalls

### Movement System
- Don't modify transforms directly if using KinematicCharacterController
- Ensure proper system execution order
- Remember that character controllers don't handle gravity automatically

### Terrain Generation
- Large terrain sizes can be very slow to generate
- Extreme parameters can create unplayable terrain
- Always validate generated terrain before using in game

### Physics Integration
- Keep kinematic vs dynamic body types consistent
- Don't mix different movement approaches on same entity
- Be careful with collider sizes and shapes

### Performance
- Debug logging can significantly impact performance
- Large textures and models affect loading times
- Too many entities can overwhelm systems

## Development Environment Setup

### Required Tools
- Rust toolchain (latest stable)
- Git with LFS extension
- Audio development libraries (libasound2-dev on Ubuntu)

### Optional Tools
- `cargo-watch` for automatic rebuilds
- `cargo-expand` for macro debugging
- Performance profilers (perf, flamegraph)

### Editor Configuration
- Configure Rust analyzer for code completion
- Set up auto-formatting on save
- Install Git LFS extension if available

## Collaboration Workflows

### Code Changes
1. **Test locally** before committing
2. **Use descriptive commit messages** explaining why, not just what
3. **Keep commits focused** - one logical change per commit
4. **Test integration** after merging

### Asset Changes
1. **Add assets to appropriate directories**
2. **Verify LFS tracking** before committing
3. **Test loading in game** before pushing
4. **Document any new asset requirements**

This workflow guide provides the foundation for productive development on the Minion project while maintaining code quality and performance.
