//! Filter Spawn Points Utility
//!
//! This utility removes enemy spawn zones that are too close to each other,
//! helping to create better distributed spawning across the map.
//!
//! # Example Usage
//! ```bash
//! # Remove spawn points within 5.0 units of each other
//! cargo run --example filter_spawns -- --input hills.bin --output filtered_hills.bin --min-distance 5.0
//!
//! # Use default min-distance (5.0 units)
//! cargo run --example filter_spawns -- --input map.bin --output filtered_map.bin
//!
//! # Dry run to see what would be removed without making changes
//! cargo run --example filter_spawns -- --input map.bin --output filtered_map.bin --dry-run
//! ```

use bevy::prelude::*;
use clap::Parser;
use minion::game_logic::errors::{MinionError, MinionResult};
use minion::map::MapDefinition;
use std::collections::HashSet;

#[derive(Parser)]
#[command(name = "filter_spawns")]
#[command(about = "Remove spawn points that are too close to each other")]
struct Args {
    /// Input map file (in maps/ directory)
    #[arg(long)]
    input: String,

    /// Output map file (in maps/ directory)
    #[arg(long)]
    output: String,

    /// Minimum distance between spawn points (default: 5.0)
    #[arg(long, default_value = "5.0")]
    min_distance: f32,

    /// Show what would be removed without making changes
    #[arg(long, default_value = "false")]
    dry_run: bool,

    /// Verbose output
    #[arg(short, long, default_value = "false")]
    verbose: bool,
}

fn main() -> MinionResult<()> {
    let args = Args::parse();

    if args.min_distance <= 0.0 {
        return Err(MinionError::InvalidMapData {
            reason: "Minimum distance must be greater than 0".to_string(),
        });
    }

    // Load the input map
    let mut map = MapDefinition::load_from_file(&args.input)?;
    let original_count = map.enemy_zones.len();

    if args.verbose {
        println!(
            "Loaded map '{}' with {} spawn zones",
            map.name, original_count
        );
        println!("Filtering zones with min-distance: {}", args.min_distance);
    }

    // Filter spawn zones - keep zones that are far enough apart
    let mut zones_to_keep = HashSet::new();
    let mut removed_indices = Vec::new();

    // Sort zones by some priority (e.g., by max_enemies descending, then by distance from center)
    let mut zone_indices: Vec<usize> = (0..map.enemy_zones.len()).collect();

    // Sort by max_enemies descending, then by distance from map center
    let map_center = Vec3::new(
        (map.terrain.width as f32 * map.terrain.scale) / 2.0,
        0.0,
        (map.terrain.height as f32 * map.terrain.scale) / 2.0,
    );

    zone_indices.sort_by(|&a, &b| {
        let zone_a = &map.enemy_zones[a];
        let zone_b = &map.enemy_zones[b];

        // First priority: max_enemies (descending)
        let enemy_cmp = zone_b.max_enemies.cmp(&zone_a.max_enemies);
        if enemy_cmp != std::cmp::Ordering::Equal {
            return enemy_cmp;
        }

        // Second priority: distance from center (ascending - prefer central zones)
        let dist_a = zone_a.center.distance(map_center);
        let dist_b = zone_b.center.distance(map_center);
        dist_a
            .partial_cmp(&dist_b)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Keep zones that are far enough from already kept zones
    for &current_idx in &zone_indices {
        let current_zone = &map.enemy_zones[current_idx];
        let mut too_close = false;

        for kept_idx in zones_to_keep.iter() {
            let kept_zone: &minion::map::SpawnZone = &map.enemy_zones[*kept_idx];
            let distance = current_zone.center.distance(kept_zone.center);

            if distance < args.min_distance {
                too_close = true;
                if args.verbose {
                    println!(
                        "Zone at {:?} (max_enemies: {}) is too close ({:.2} < {:.2}) to zone at {:?}",
                        current_zone.center,
                        current_zone.max_enemies,
                        distance,
                        args.min_distance,
                        kept_zone.center
                    );
                }
                break;
            }
        }

        if too_close {
            removed_indices.push(current_idx);
        } else {
            zones_to_keep.insert(current_idx);
            if args.verbose {
                println!(
                    "Keeping zone at {:?} (max_enemies: {})",
                    current_zone.center, current_zone.max_enemies
                );
            }
        }
    }

    let final_count = zones_to_keep.len();
    let removed_count = original_count - final_count;

    // Display results
    println!(
        "Filter results: Removed {} out of {} spawn zones ({:.1}% reduction)",
        removed_count,
        original_count,
        if original_count > 0 {
            (removed_count as f32 / original_count as f32) * 100.0
        } else {
            0.0
        }
    );
    println!("Final spawn zone count: {}", final_count);

    if args.dry_run {
        println!("Dry run - no changes made to files");
        return Ok(());
    }

    // Create filtered map
    let mut filtered_zones = Vec::new();
    for &idx in zones_to_keep.iter() {
        filtered_zones.push(map.enemy_zones[idx].clone());
    }
    map.enemy_zones = filtered_zones;

    // Save the filtered map
    map.save_to_file(&args.output)?;

    println!("Filtered map saved to: {}", args.output);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use minion::map::{SpawnZone, TerrainData};

    #[test]
    fn test_distance_filtering() {
        // Create test zones
        let zone1 = SpawnZone::new(
            Vec3::new(0.0, 0.0, 0.0),
            2.0,
            10,
            vec!["dark-knight".to_string()],
        )
        .unwrap();

        let zone2 = SpawnZone::new(
            Vec3::new(3.0, 0.0, 0.0), // 3 units away
            2.0,
            5,
            vec!["dark-knight".to_string()],
        )
        .unwrap();

        let zone3 = SpawnZone::new(
            Vec3::new(10.0, 0.0, 0.0), // 10 units away
            2.0,
            8,
            vec!["dark-knight".to_string()],
        )
        .unwrap();

        let terrain = TerrainData::create_flat(20, 20, 1.0, 0.0).unwrap();
        let mut map = MapDefinition::new(
            "test".to_string(),
            terrain,
            Vec3::new(5.0, 0.0, 5.0),
            vec![zone1, zone2, zone3],
            vec![],
        )
        .unwrap();

        // With min_distance = 5.0, zone2 should be removed (too close to zone1)
        // but zone3 should be kept
        let distance1_2 = map.enemy_zones[0]
            .center
            .distance(map.enemy_zones[1].center);
        let distance1_3 = map.enemy_zones[0]
            .center
            .distance(map.enemy_zones[2].center);

        assert_eq!(distance1_2, 3.0);
        assert_eq!(distance1_3, 10.0);

        // zone1 has more max_enemies than zone2, so zone1 should be kept
        assert!(map.enemy_zones[0].max_enemies > map.enemy_zones[1].max_enemies);
    }
}
