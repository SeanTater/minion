use crate::game_logic::errors::{MinionError, MinionResult};
use crate::map::{MapDefinition, SpawnZone, TerrainData};
use crate::pathfinding::{NavigationGrid, PathfindingConfig};
use crate::resources::{GameConfig, GameState};
use crate::terrain::coordinates::get_height_at_world_interpolated;
use crate::terrain_generation::{get_terrain_preset, is_suitable_for_spawning};
use bevy::prelude::*;

pub struct MapLoaderPlugin;

impl Plugin for MapLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), load_map);
    }
}

pub fn load_map(mut commands: Commands, game_config: Res<GameConfig>) {
    let map_result = load_map_from_config(&game_config);

    match map_result {
        Ok(map) => {
            info!("Successfully loaded map: {}", map.name);

            // Initialize NavigationGrid from the terrain data and environment objects
            let pathfinding_config = PathfindingConfig::default();
            match NavigationGrid::from_terrain_and_objects(
                &map.terrain,
                &map.environment_objects,
                pathfinding_config,
            ) {
                Ok(nav_grid) => {
                    info!(
                        "Successfully created navigation grid ({width}x{height}) with {obj_count} environment objects",
                        width = nav_grid.width,
                        height = nav_grid.height,
                        obj_count = map.environment_objects.len()
                    );
                    commands.insert_resource(nav_grid);
                }
                Err(err) => {
                    warn!("Failed to create navigation grid: {err}");
                    warn!("Pathfinding will not be available - falling back to direct movement");
                }
            }

            commands.insert_resource(map);
        }
        Err(err) => {
            warn!("Failed to load map: {err}");

            // Attempt progressive degradation based on error type
            let fallback_result = create_fallback_map_progressive(&game_config, &err);

            match fallback_result {
                Ok(fallback_map) => {
                    info!("Successfully created fallback map: {}", fallback_map.name);

                    // Initialize NavigationGrid for fallback map too
                    let pathfinding_config = PathfindingConfig::default();
                    match NavigationGrid::from_terrain_and_objects(
                        &fallback_map.terrain,
                        &fallback_map.environment_objects,
                        pathfinding_config,
                    ) {
                        Ok(nav_grid) => {
                            info!(
                                "Successfully created navigation grid for fallback map ({width}x{height}) with {obj_count} environment objects",
                                width = nav_grid.width,
                                height = nav_grid.height,
                                obj_count = fallback_map.environment_objects.len()
                            );
                            commands.insert_resource(nav_grid);
                        }
                        Err(err) => {
                            warn!("Failed to create navigation grid for fallback map: {err}");
                        }
                    }

                    commands.insert_resource(fallback_map);
                }
                Err(fallback_err) => {
                    error!("Failed to create fallback map: {fallback_err}");
                    warn!("Using minimal hardcoded fallback...");

                    // Last resort: hardcoded fallback
                    let minimal_map = create_minimal_fallback_map();

                    // Initialize NavigationGrid for minimal fallback too
                    let pathfinding_config = PathfindingConfig::default();
                    match NavigationGrid::from_terrain_and_objects(
                        &minimal_map.terrain,
                        &minimal_map.environment_objects,
                        pathfinding_config,
                    ) {
                        Ok(nav_grid) => {
                            info!(
                                "Successfully created navigation grid for minimal fallback ({width}x{height}) with {obj_count} environment objects",
                                width = nav_grid.width,
                                height = nav_grid.height,
                                obj_count = minimal_map.environment_objects.len()
                            );
                            commands.insert_resource(nav_grid);
                        }
                        Err(err) => {
                            warn!("Failed to create navigation grid for minimal fallback: {err}");
                        }
                    }

                    commands.insert_resource(minimal_map);
                }
            }
        }
    }
}

fn load_map_from_config(game_config: &GameConfig) -> MinionResult<MapDefinition> {
    let map_file = &game_config.settings.map_file_path;
    info!("Attempting to load map from: {map_file}");

    // Enhanced error handling with more specific error types
    let result = MapDefinition::load_from_file(map_file);

    match &result {
        Ok(_) => debug!("Map loaded successfully from {map_file}"),
        Err(err) => {
            debug!("Map loading failed with error: {err}");
            match err {
                MinionError::MapFileNotFound { path } => {
                    warn!(
                        "Map file not found: {}. Check that the file exists and is readable.",
                        path.display()
                    );
                }
                MinionError::InvalidMapData { reason } => {
                    warn!(
                        "Map data is invalid: {reason}. The file may be corrupted or from an incompatible version."
                    );
                }
                MinionError::ConfigDirCreationFailed(_) => {
                    warn!("Failed to access maps directory. Check file permissions.");
                }
                _ => warn!("Unexpected error loading map: {err}"),
            }
        }
    }

    result
}

/// Create a fallback map using progressive degradation based on the specific error
fn create_fallback_map_progressive(
    game_config: &GameConfig,
    error: &MinionError,
) -> MinionResult<MapDefinition> {
    info!("Creating progressive fallback map for error type: {error}");

    match error {
        MinionError::MapFileNotFound { path } => {
            info!("Map file not found, generating procedural terrain as fallback");
            warn!(
                "SOLUTION: Create a map file at {} or check that the path in config is correct",
                path.display()
            );
            warn!("You can generate maps using: cargo run --bin map_generator");
            create_generated_terrain_map(game_config)
        }
        MinionError::CorruptedMapFile { reason } => {
            info!("Map file corrupted, generating fresh terrain");
            warn!("SOLUTION: The map file appears to be corrupted ({reason})");
            warn!("Try regenerating the map file or restore from backup");
            create_generated_terrain_map(game_config)
        }
        MinionError::InvalidTerrainData { reason } => {
            info!(
                "Invalid terrain data, using procedural terrain but keeping other map elements if possible"
            );
            warn!("SOLUTION: Terrain data issue ({reason})");
            warn!("The terrain will be regenerated but other map elements may be preserved");
            create_mixed_fallback_map(game_config)
        }
        MinionError::InvalidSpawnZoneData { reason } => {
            info!("Invalid spawn zones, regenerating spawn zones on existing terrain");
            warn!("SOLUTION: Spawn zone issue ({reason})");
            warn!("Spawn zones will be regenerated based on terrain analysis");
            create_respawn_fallback_map(game_config)
        }
        MinionError::MapValidationFailed { reason } => {
            info!("Map validation failed, attempting to fix automatically");
            warn!("SOLUTION: Map validation error ({reason})");
            warn!("Attempting automatic correction or falling back to procedural generation");
            create_generated_terrain_map(game_config)
        }
        _ => {
            info!("General map error, creating full procedural fallback");
            warn!("SOLUTION: Unexpected map error, using procedural fallback");
            warn!("Check game logs for more details about the specific issue");
            create_generated_terrain_map(game_config)
        }
    }
}

/// Create a map with generated terrain and intelligent spawn zone placement
fn create_generated_terrain_map(_game_config: &GameConfig) -> MinionResult<MapDefinition> {
    // Generate procedural terrain based on a preset
    // TODO: Could use game_config to customize terrain generation parameters
    let terrain_generator =
        get_terrain_preset("hills", Some(42)).ok_or_else(|| MinionError::InvalidMapData {
            reason: "Failed to get terrain preset".to_string(),
        })?;

    let terrain = terrain_generator.generate(32, 32, 0.5)?;

    // Generate spawn zones using terrain analysis
    let spawn_zones = generate_terrain_based_spawn_zones(&terrain)?;

    // Find suitable player spawn position
    let player_spawn = find_suitable_player_spawn(&terrain)?;

    MapDefinition::new(
        "procedural_fallback".to_string(),
        terrain,
        player_spawn,
        spawn_zones,
        vec![], // No environment objects for fallback
    )
}

/// Create a map with mixed fallback (some procedural, some hardcoded)
fn create_mixed_fallback_map(_game_config: &GameConfig) -> MinionResult<MapDefinition> {
    // For this implementation, fall back to generated terrain
    // Could be enhanced to attempt partial map recovery
    create_minimal_fallback_map_result()
}

/// Create a map with regenerated spawn zones on basic terrain
fn create_respawn_fallback_map(_game_config: &GameConfig) -> MinionResult<MapDefinition> {
    let terrain = TerrainData::create_flat(24, 24, 1.5, 0.0)?;
    let spawn_zones = generate_terrain_based_spawn_zones(&terrain)?;
    let player_spawn = Vec3::new(0.0, 1.0, 0.0);

    MapDefinition::new(
        "respawn_fallback".to_string(),
        terrain,
        player_spawn,
        spawn_zones,
        vec![],
    )
}

/// Generate spawn zones based on terrain analysis
fn generate_terrain_based_spawn_zones(terrain: &TerrainData) -> MinionResult<Vec<SpawnZone>> {
    let mut spawn_zones = Vec::new();
    let max_slope = 0.3; // Maximum slope for spawning
    let min_distance = 8.0; // Minimum distance between spawn zones
    let max_attempts = 100; // Prevent infinite loops

    // Calculate terrain bounds in world coordinates
    let terrain_width = terrain.width as f32 * terrain.scale;
    let terrain_height = terrain.height as f32 * terrain.scale;

    let mut attempts = 0;
    let target_zones = 5; // Try to create 5 spawn zones

    while spawn_zones.len() < target_zones && attempts < max_attempts {
        attempts += 1;

        // Generate random position within terrain bounds (with margin)
        let margin = 4.0;
        let x = (rand::random::<f32>() - 0.5) * (terrain_width - margin * 2.0);
        let z = (rand::random::<f32>() - 0.5) * (terrain_height - margin * 2.0);

        // Check if position is suitable
        if !is_suitable_for_spawning(terrain, x, z, max_slope) {
            continue;
        }

        // Check distance from existing spawn zones
        let too_close = spawn_zones.iter().any(|zone: &SpawnZone| {
            let distance = (Vec3::new(x, 0.0, z) - zone.center).length();
            distance < min_distance
        });

        if too_close {
            continue;
        }

        // Get terrain height for this position
        let height = get_height_at_world_interpolated(terrain, x, z).unwrap_or(0.0);

        // Create spawn zone
        let spawn_zone = SpawnZone::new(
            Vec3::new(x, height, z),
            3.0, // radius
            2,   // max enemies
            vec!["dark-knight".to_string()],
        )?;

        spawn_zones.push(spawn_zone);
        debug!("Generated spawn zone at ({x:.1}, {height:.1}, {z:.1})");
    }

    // If we couldn't generate enough zones, add some basic fallback zones
    if spawn_zones.is_empty() {
        warn!("Failed to generate terrain-based spawn zones, using basic fallback positions");
        spawn_zones = create_basic_spawn_zones(terrain)?;
    }

    info!(
        "Generated {} spawn zones using terrain analysis",
        spawn_zones.len()
    );
    Ok(spawn_zones)
}

/// Create basic spawn zones as a last resort
fn create_basic_spawn_zones(terrain: &TerrainData) -> MinionResult<Vec<SpawnZone>> {
    let base_height = terrain.heights.iter().sum::<f32>() / terrain.heights.len() as f32;

    let positions = vec![
        Vec3::new(6.0, base_height, 6.0),
        Vec3::new(-6.0, base_height, 6.0),
        Vec3::new(6.0, base_height, -6.0),
        Vec3::new(-6.0, base_height, -6.0),
        Vec3::new(0.0, base_height, 10.0),
    ];

    let mut spawn_zones = Vec::new();
    for (i, pos) in positions.into_iter().enumerate() {
        let zone = SpawnZone::new(pos, 2.5, 1, vec!["dark-knight".to_string()]).map_err(|e| {
            MinionError::InvalidSpawnZoneData {
                reason: format!("Failed to create basic spawn zone {i}: {e}"),
            }
        })?;
        spawn_zones.push(zone);
    }

    Ok(spawn_zones)
}

/// Find a suitable player spawn position on the terrain
fn find_suitable_player_spawn(terrain: &TerrainData) -> MinionResult<Vec3> {
    let max_slope = 0.2; // Even stricter slope requirement for player
    let max_attempts = 50;

    // Try center first
    if is_suitable_for_spawning(terrain, 0.0, 0.0, max_slope) {
        let height = get_height_at_world_interpolated(terrain, 0.0, 0.0).unwrap_or(0.0);
        return Ok(Vec3::new(0.0, height + 1.0, 0.0));
    }

    // Search in expanding circles from center
    for attempt in 0..max_attempts {
        let radius = (attempt as f32 + 1.0) * 2.0;
        let angle = (attempt as f32) * 0.7; // Arbitrary step to avoid patterns

        let x = radius * angle.cos();
        let z = radius * angle.sin();

        if is_suitable_for_spawning(terrain, x, z, max_slope) {
            let height = get_height_at_world_interpolated(terrain, x, z).unwrap_or(0.0);
            return Ok(Vec3::new(x, height + 1.0, z));
        }
    }

    // Fallback to a basic position
    warn!("Could not find suitable player spawn position, using fallback");
    Ok(Vec3::new(0.0, 2.0, 0.0))
}

/// Create the minimal hardcoded fallback map (original behavior)
fn create_minimal_fallback_map() -> MapDefinition {
    create_minimal_fallback_map_result().expect("Failed to create minimal fallback map")
}

/// Create the minimal hardcoded fallback map with error handling
fn create_minimal_fallback_map_result() -> MinionResult<MapDefinition> {
    // Create flat terrain matching the current hardcoded ground plane (20x20)
    let terrain = TerrainData::create_flat(20, 20, 1.0, 0.0)?;

    // Create spawn zones that match the current hardcoded enemy positions
    let spawn_zones = vec![
        SpawnZone::new(
            Vec3::new(5.0, 0.0, 5.0),
            2.0,
            1,
            vec!["dark-knight".to_string()],
        )?,
        SpawnZone::new(
            Vec3::new(-5.0, 0.0, 5.0),
            2.0,
            1,
            vec!["dark-knight".to_string()],
        )?,
        SpawnZone::new(
            Vec3::new(5.0, 0.0, -5.0),
            2.0,
            1,
            vec!["dark-knight".to_string()],
        )?,
        SpawnZone::new(
            Vec3::new(-5.0, 0.0, -5.0),
            2.0,
            1,
            vec!["dark-knight".to_string()],
        )?,
        SpawnZone::new(
            Vec3::new(0.0, 0.0, 8.0),
            2.0,
            1,
            vec!["dark-knight".to_string()],
        )?,
    ];

    MapDefinition::new(
        "minimal_fallback".to_string(),
        terrain,
        Vec3::new(0.0, 1.0, 0.0), // Player spawn position - matches current hardcoded
        spawn_zones,
        vec![], // No environment objects for now
    )
}
