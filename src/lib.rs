pub mod components;
pub mod config;
pub mod game_logic;
pub mod map;
pub mod pathfinding;
pub mod plugins;
pub mod resources;
pub mod terrain;
pub mod terrain_generation;

// Selective re-exports for external consumers

// Plugins - main.rs needs all plugins
pub use plugins::*;

// Game logic - examples need errors and some core types
pub use game_logic::errors::{MinionError, MinionResult};

// Map - examples need core map types
pub use map::{EnvironmentObject, MapDefinition, SpawnZone, TerrainData};

// Terrain generation - examples need utility functions
pub use terrain_generation::is_suitable_for_spawning;
