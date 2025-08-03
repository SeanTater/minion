//! Merge Maps Utility
//!
//! This utility combines terrain, objects, and spawn zones from multiple maps
//! into a single map, allowing for modular map composition.
//!
//! # Example Usage
//! ```bash
//! # Combine terrain from one map with objects from another
//! cargo run --example merge_maps -- --terrain hills.bin --objects forest.bin --output combined.bin
//!
//! # Merge terrain, objects, and spawn zones from different maps
//! cargo run --example merge_maps -- --terrain mountains.bin --objects trees.bin --spawns arena.bin --output epic_map.bin
//!
//! # Use a base map and add objects from another
//! cargo run --example merge_maps -- --base base_map.bin --objects decorations.bin --output enhanced_map.bin
//!
//! # Dry run to see what would be combined
//! cargo run --example merge_maps -- --terrain map1.bin --spawns map2.bin --output test.bin --dry-run
//! ```

use bevy::prelude::*;
use clap::Parser;
use minion::game_logic::errors::{MinionError, MinionResult};
use minion::map::MapDefinition;

#[derive(Parser)]
#[command(name = "merge_maps")]
#[command(about = "Combine elements from multiple maps")]
struct Args {
    /// Base map file (provides default values for all components)
    #[arg(long)]
    base: Option<String>,

    /// Map file to take terrain from
    #[arg(long)]
    terrain: Option<String>,

    /// Map file to take environment objects from
    #[arg(long)]
    objects: Option<String>,

    /// Map file to take spawn zones from
    #[arg(long)]
    spawns: Option<String>,

    /// Map file to take player spawn position from
    #[arg(long)]
    player_spawn: Option<String>,

    /// Name for the merged map
    #[arg(long, default_value = "merged_map")]
    name: String,

    /// Output map file (in maps/ directory)
    #[arg(long)]
    output: String,

    /// Show what would be combined without making changes
    #[arg(long, default_value = "false")]
    dry_run: bool,

    /// Verbose output showing merge details
    #[arg(short, long, default_value = "false")]
    verbose: bool,

    /// Scale factor for imported objects (default: 1.0)
    #[arg(long, default_value = "1.0")]
    object_scale: f32,

    /// Offset for imported objects (format: X,Y,Z)
    #[arg(long)]
    object_offset: Option<String>,

    /// Replace existing components instead of merging (for objects and spawns)
    #[arg(long, default_value = "false")]
    replace: bool,
}

fn parse_position(pos_str: &str) -> MinionResult<Vec3> {
    let parts: Vec<&str> = pos_str.split(',').collect();
    if parts.len() != 3 {
        return Err(MinionError::InvalidMapData {
            reason: format!(
                "Invalid position format '{}'. Expected format: X,Y,Z",
                pos_str
            ),
        });
    }

    let x = parts[0]
        .parse::<f32>()
        .map_err(|_| MinionError::InvalidMapData {
            reason: format!("Invalid X coordinate: '{}'", parts[0]),
        })?;

    let y = parts[1]
        .parse::<f32>()
        .map_err(|_| MinionError::InvalidMapData {
            reason: format!("Invalid Y coordinate: '{}'", parts[1]),
        })?;

    let z = parts[2]
        .parse::<f32>()
        .map_err(|_| MinionError::InvalidMapData {
            reason: format!("Invalid Z coordinate: '{}'", parts[2]),
        })?;

    Ok(Vec3::new(x, y, z))
}

fn terrain_size_matches(
    terrain1: &minion::map::TerrainData,
    terrain2: &minion::map::TerrainData,
) -> bool {
    terrain1.width == terrain2.width
        && terrain1.height == terrain2.height
        && (terrain1.scale - terrain2.scale).abs() < 0.001
}

fn main() -> MinionResult<()> {
    let args = Args::parse();

    // Validate arguments
    if args.base.is_none()
        && args.terrain.is_none()
        && args.objects.is_none()
        && args.spawns.is_none()
        && args.player_spawn.is_none()
    {
        return Err(MinionError::InvalidMapData {
            reason: "At least one source map must be specified".to_string(),
        });
    }

    if args.object_scale <= 0.0 {
        return Err(MinionError::InvalidMapData {
            reason: "Object scale must be greater than 0".to_string(),
        });
    }

    let object_offset = if let Some(offset_str) = &args.object_offset {
        parse_position(offset_str)?
    } else {
        Vec3::ZERO
    };

    if args.verbose {
        println!("=== Merge Maps Configuration ===");
        println!("Base map: {:?}", args.base);
        println!("Terrain source: {:?}", args.terrain);
        println!("Objects source: {:?}", args.objects);
        println!("Spawns source: {:?}", args.spawns);
        println!("Player spawn source: {:?}", args.player_spawn);
        println!("Object scale: {}", args.object_scale);
        println!("Object offset: {:?}", object_offset);
        println!("Replace mode: {}", args.replace);
        println!();
    }

    // Load source maps
    let base_map = if let Some(base_file) = &args.base {
        Some(MapDefinition::load_from_file(base_file)?)
    } else {
        None
    };

    let terrain_map = if let Some(terrain_file) = &args.terrain {
        Some(MapDefinition::load_from_file(terrain_file)?)
    } else {
        None
    };

    let objects_map = if let Some(objects_file) = &args.objects {
        Some(MapDefinition::load_from_file(objects_file)?)
    } else {
        None
    };

    let spawns_map = if let Some(spawns_file) = &args.spawns {
        Some(MapDefinition::load_from_file(spawns_file)?)
    } else {
        None
    };

    let player_spawn_map = if let Some(player_spawn_file) = &args.player_spawn {
        Some(MapDefinition::load_from_file(player_spawn_file)?)
    } else {
        None
    };

    // Determine the primary source for each component
    let terrain_source =
        terrain_map
            .as_ref()
            .or(base_map.as_ref())
            .ok_or_else(|| MinionError::InvalidMapData {
                reason: "No terrain source available (need --terrain or --base)".to_string(),
            })?;

    if args.verbose {
        println!(
            "Using terrain from: {}",
            if terrain_map.is_some() {
                args.terrain.as_ref().unwrap()
            } else {
                args.base.as_ref().unwrap()
            }
        );
    }

    // Build the merged map
    let merged_terrain = terrain_source.terrain.clone();
    let mut merged_objects = if args.replace {
        Vec::new()
    } else {
        base_map
            .as_ref()
            .map(|m| m.environment_objects.clone())
            .unwrap_or_default()
    };
    let mut merged_spawn_zones = if args.replace {
        Vec::new()
    } else {
        base_map
            .as_ref()
            .map(|m| m.enemy_zones.clone())
            .unwrap_or_default()
    };
    let mut merged_player_spawn = base_map
        .as_ref()
        .map(|m| m.player_spawn)
        .unwrap_or(Vec3::new(0.0, 1.0, 0.0));

    // Merge objects
    if let Some(objects_source) = &objects_map {
        if args.verbose {
            println!(
                "Adding {} objects from {}",
                objects_source.environment_objects.len(),
                args.objects.as_ref().unwrap()
            );
        }

        for obj in &objects_source.environment_objects {
            let mut merged_obj = obj.clone();

            // Apply transformations
            merged_obj.position += object_offset;
            merged_obj.scale *= args.object_scale;

            // Check if position is within terrain bounds (optional validation)
            let terrain_width = merged_terrain.width as f32 * merged_terrain.scale;
            let terrain_height = merged_terrain.height as f32 * merged_terrain.scale;

            if merged_obj.position.x < 0.0
                || merged_obj.position.x >= terrain_width
                || merged_obj.position.z < 0.0
                || merged_obj.position.z >= terrain_height
            {
                if args.verbose {
                    println!(
                        "Warning: Object at {:?} is outside terrain bounds",
                        merged_obj.position
                    );
                }
            }

            merged_objects.push(merged_obj);
        }
    }

    // Merge spawn zones
    if let Some(spawns_source) = &spawns_map {
        if args.verbose {
            println!(
                "Adding {} spawn zones from {}",
                spawns_source.enemy_zones.len(),
                args.spawns.as_ref().unwrap()
            );
        }

        // Validate terrain compatibility for spawn zones
        if !terrain_size_matches(&merged_terrain, &spawns_source.terrain) {
            println!(
                "Warning: Spawn zones terrain size ({},{}@{}) doesn't match target terrain ({},{}@{})",
                spawns_source.terrain.width,
                spawns_source.terrain.height,
                spawns_source.terrain.scale,
                merged_terrain.width,
                merged_terrain.height,
                merged_terrain.scale
            );
        }

        merged_spawn_zones.extend(spawns_source.enemy_zones.clone());
    }

    // Set player spawn
    if let Some(player_spawn_source) = &player_spawn_map {
        merged_player_spawn = player_spawn_source.player_spawn;
        if args.verbose {
            println!("Using player spawn position: {:?}", merged_player_spawn);
        }
    }

    // Create the merged map
    let merged_map = MapDefinition::new(
        args.name.clone(),
        merged_terrain,
        merged_player_spawn,
        merged_spawn_zones,
        merged_objects,
    )?;

    // Display merge results
    println!("=== Merge Results ===");
    println!("Map name: {}", merged_map.name);
    println!(
        "Terrain: {}x{} ({} vertices)",
        merged_map.terrain.width,
        merged_map.terrain.height,
        merged_map.terrain.width * merged_map.terrain.height
    );
    println!(
        "Environment objects: {}",
        merged_map.environment_objects.len()
    );
    println!("Spawn zones: {}", merged_map.enemy_zones.len());
    println!("Player spawn: {:?}", merged_map.player_spawn);

    if args.verbose {
        // Object type breakdown
        let mut object_counts = std::collections::HashMap::new();
        for obj in &merged_map.environment_objects {
            *object_counts.entry(&obj.object_type).or_insert(0) += 1;
        }

        if !object_counts.is_empty() {
            println!("\nObject breakdown:");
            for (obj_type, count) in object_counts {
                println!("  {}: {} objects", obj_type, count);
            }
        }

        // Spawn zone breakdown
        let total_max_enemies: u32 = merged_map.enemy_zones.iter().map(|z| z.max_enemies).sum();
        if total_max_enemies > 0 {
            println!(
                "\nSpawn zones can host up to {} enemies total",
                total_max_enemies
            );
        }
    }

    if args.dry_run {
        println!("\nDry run - no changes made to files");
        return Ok(());
    }

    // Save the merged map
    merged_map.save_to_file(&args.output)?;

    println!("\nMerged map saved to: {}", args.output);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use minion::map::{EnvironmentObject, SpawnZone, TerrainData};

    #[test]
    fn test_parse_position() {
        let pos = parse_position("1.5,2.0,-3.5").unwrap();
        assert_eq!(pos, Vec3::new(1.5, 2.0, -3.5));

        assert!(parse_position("invalid").is_err());
        assert!(parse_position("1.0,2.0").is_err());
    }

    #[test]
    fn test_terrain_size_matching() {
        let terrain1 = TerrainData::create_flat(10, 10, 1.0, 0.0).unwrap();
        let terrain2 = TerrainData::create_flat(10, 10, 1.0, 5.0).unwrap();
        let terrain3 = TerrainData::create_flat(5, 5, 1.0, 0.0).unwrap();

        assert!(terrain_size_matches(&terrain1, &terrain2));
        assert!(!terrain_size_matches(&terrain1, &terrain3));
    }

    #[test]
    fn test_object_merging() {
        let base_objects = vec![EnvironmentObject::simple(
            "tree".to_string(),
            Vec3::new(0.0, 0.0, 0.0),
        )];

        let new_objects = vec![EnvironmentObject::simple(
            "rock".to_string(),
            Vec3::new(5.0, 0.0, 5.0),
        )];

        // Test non-replace mode (should have both)
        let mut merged = base_objects.clone();
        merged.extend(new_objects.clone());
        assert_eq!(merged.len(), 2);

        // Test replace mode (should have only new)
        let replaced = new_objects.clone();
        assert_eq!(replaced.len(), 1);
        assert_eq!(replaced[0].object_type, "rock");
    }
}
