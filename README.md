# Minion

A Diablo-like action RPG built with Rust and Bevy 0.14.

## Features

- **Main Menu UI**: Username selection with persistent local configuration
- **User Profiles**: Automatic saving of username and score to `~/.config/minion/config.toml`
- **3D Isometric View**: Classic ARPG camera angle with smooth camera following
- **Click-to-Move**: Point and click movement system
- **Ranged Combat**: Right-click projectile system with enemy collision
- **Area Effects**: Spacebar-activated spells with multiple effect types
- **Enemy AI**: Intelligent enemy spawning, movement, and respawning system
- **Score System**: Real-time score tracking with persistent high scores
- **PBR Rendering**: Modern lighting with shadows and visual effects
- **Linux/Wayland Support**: Optimized for Linux desktop

## Controls

### Main Menu
- **Type**: Enter your username (alphanumeric characters and hyphens)
- **Start Button**: Begin the game
- **Escape**: Exit game

### In-Game
- **Left Click**: Move character to clicked location
- **Right Click**: Fire projectile at target location
- **Spacebar**: Create area effect at player location
- **Tab**: Cycle between area effect types (Magic: blue, Poison: green)

## Area Effects

- **Magic**: Blue effect, 3.0 radius, 150 DPS, 2-second duration
- **Poison**: Green effect, 4.0 radius, 80 DPS, 4-second duration

## Scoring

- **10 points** per enemy defeated (via bullets or area effects)
- Scores are automatically saved and persist between game sessions

## Running

```bash
cargo run
```

## Requirements

- Rust 2024 edition
- ALSA development libraries (`sudo apt-get install libasound2-dev`)

## Configuration

Game configuration is automatically saved to:
- **Linux**: `~/.config/minion/config.toml`

The config file stores:
- Username
- High score