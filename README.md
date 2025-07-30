# Minion

A Diablo-like action RPG built with Rust and Bevy 0.16, featuring 3D isometric gameplay, combat systems, and persistent user profiles.

## Features

### Core Gameplay
- **3D Isometric View**: Classic ARPG camera angle with smooth following
- **Click-to-Move**: Point and click movement system with ground-plane raycasting
- **Ranged Combat**: Right-click projectile system with collision detection
- **Area Effects**: Spacebar-activated spells with multiple effect types (Magic/Poison)
- **Enemy AI**: Intelligent spawning, pathfinding, and respawning system

### User Experience
- **Main Menu UI**: Clean username selection interface
- **User Profiles**: Persistent configuration saved to `~/.config/minion/config.toml`
- **Score System**: Real-time tracking with persistent high scores
- **Tooltips**: Interactive UI elements with helpful information

### Technical Features
- **PBR Rendering**: Modern lighting with shadows and visual effects
- **Linux/Wayland Support**: Optimized for Linux desktop environments
- **Modular Architecture**: Well-organized plugin system for maintainability
- **Configuration Management**: Automatic config loading/saving with TOML

## Controls

### Main Menu
- **Type**: Enter username (alphanumeric and hyphens)
- **Start Button**: Begin the game
- **Escape**: Exit application

### In-Game
- **Left Click**: Move character to target location
- **Right Click**: Fire projectile toward cursor
- **Spacebar**: Create area effect at player position
- **Tab**: Cycle between area effect types
- **Escape**: Return to main menu

## Combat System

### Projectiles
- **Damage**: 2 HP per hit
- **Speed**: 15 units/second
- **Lifetime**: 3 seconds
- **Collision**: 0.6 unit radius

### Area Effects

| Type | Color | Radius | DPS | Duration |
|------|-------|--------|-----|----------|
| Magic | Blue | 3.0 | 150 | 2s |
| Poison | Green | 4.0 | 80 | 4s |

### Enemies
- **Health**: 3 HP
- **Speed**: 3 units/second
- **Chase Distance**: 8 units
- **Spawn Range**: 5-10.5 units from player
- **Respawn**: Automatic when eliminated

## Scoring

- **10 points** per enemy defeated (projectiles or area effects)
- Scores automatically saved and persist between sessions
- High score tracking per user profile

## Architecture

### Project Structure
```
src/
├── components/          # ECS components and shared types
├── config.rs           # Configuration file handling
├── game_logic/         # Core game mechanics
│   ├── damage.rs       # Damage calculation systems
│   ├── names.rs        # Name generation utilities
│   └── spawning.rs     # Entity spawning logic
├── plugins/            # Bevy plugin modules
│   ├── combat.rs       # Projectile and area effect systems
│   ├── enemy.rs        # Enemy AI and behavior
│   ├── player.rs       # Player movement and input
│   ├── scene.rs        # 3D scene setup and lighting
│   ├── tooltips.rs     # UI tooltip system
│   └── ui.rs           # Main menu and HUD
├── resources/          # Global game resources and state
├── lib.rs             # Library exports
└── main.rs            # Application entry point
```

### Key Systems

#### Input Processing
1. Mouse raycast to ground plane (y=0)
2. Movement target set on Player component
3. Smooth interpolation with 0.1 unit arrival threshold

#### Camera System
- Fixed isometric offset: (10, 15, 10)
- Always looks at player position
- Smooth following with consistent framing

#### Combat Flow
- Projectiles: Click → spawn → physics → collision → damage
- Area Effects: Spacebar → spawn at player → damage over time → despawn

#### Enemy Behavior
- Spawn randomly around player (5-10.5 unit ring)
- Chase player when within 8 units
- Basic collision avoidance
- Automatic respawning system

## Development

### Requirements
- Rust 2024 edition
- ALSA development libraries: `sudo apt-get install libasound2-dev`

### Commands
```bash
# Run the game
cargo run

# Development tools
cargo build --release
cargo check
cargo fmt
cargo clippy
```

### Configuration Storage
- **Linux**: `~/.config/minion/config.toml`
- Contains username and high score
- Automatically created on first run

## Dependencies

- **bevy**: 0.16 (game engine with Wayland support)
- **bevy_lunex**: 0.4.1 (UI framework)
- **serde**: 1.0 (serialization for config)
- **toml**: 0.8 (config file format)
- **dirs**: 5.0 (cross-platform config directories)
- **rand**: 0.8 (random number generation)

## License

See [LICENSE.md](LICENSE.md) for details.