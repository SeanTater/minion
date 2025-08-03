//! Map Information Utility
//!
//! This utility displays comprehensive information and statistics about map files,
//! helping with map analysis and debugging.
//!
//! # Example Usage
//! ```bash
//! # Basic map information
//! cargo run --example map_info -- --input complex_map.bin
//!
//! # Detailed information with verbose output
//! cargo run --example map_info -- --input hills.bin --verbose
//!
//! # Show only terrain information
//! cargo run --example map_info -- --input map.bin --section terrain
//!
//! # Export information to JSON format
//! cargo run --example map_info -- --input map.bin --format json --output map_info.json
//! ```

use bevy::prelude::*;
use clap::Parser;
use minion::game_logic::errors::{MinionError, MinionResult};
use minion::map::MapDefinition;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Parser)]
#[command(name = "map_info")]
#[command(about = "Display detailed map information and statistics")]
struct Args {
    /// Input map file (in maps/ directory)
    #[arg(long)]
    input: String,

    /// Verbose output with detailed breakdowns
    #[arg(short, long, default_value = "false")]
    verbose: bool,

    /// Output format: text, json
    #[arg(long, default_value = "text")]
    format: String,

    /// Output file for exporting information (optional)
    #[arg(long)]
    output: Option<String>,

    /// Show only specific section: terrain, spawns, objects, all
    #[arg(long, default_value = "all")]
    section: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct MapStats {
    name: String,
    file_size_bytes: u64,
    terrain: TerrainStats,
    spawn_zones: SpawnZoneStats,
    environment_objects: ObjectStats,
}

#[derive(Debug, Serialize, Deserialize)]
struct TerrainStats {
    dimensions: (u32, u32),
    scale: f32,
    world_size: (f32, f32),
    total_vertices: u32,
    height_range: (f32, f32),
    mean_height: f32,
    std_deviation: f32,
    height_distribution: Vec<(String, u32)>, // height ranges with counts
}

#[derive(Debug, Serialize, Deserialize)]
struct SpawnZoneStats {
    total_zones: usize,
    total_max_enemies: u32,
    average_radius: f32,
    radius_range: (f32, f32),
    enemy_types: HashMap<String, u32>,     // type -> count
    zone_distribution: Vec<(String, u32)>, // radius ranges with counts
}

#[derive(Debug, Serialize, Deserialize)]
struct ObjectStats {
    total_objects: usize,
    object_types: HashMap<String, u32>, // type -> count
    position_bounds: ((f32, f32, f32), (f32, f32, f32)), // min, max
    scale_range: (f32, f32),
    mean_scale: f32,
}

impl TerrainStats {
    fn analyze(terrain: &minion::map::TerrainData) -> Self {
        let mut min_height = f32::INFINITY;
        let mut max_height = f32::NEG_INFINITY;
        let mut sum = 0.0;
        let mut sum_squares = 0.0;

        for &height in &terrain.heights {
            min_height = min_height.min(height);
            max_height = max_height.max(height);
            sum += height;
            sum_squares += height * height;
        }

        let count = terrain.heights.len() as f32;
        let mean = sum / count;
        let variance = (sum_squares / count) - (mean * mean);
        let std_dev = variance.sqrt();

        // Create height distribution buckets
        let range = max_height - min_height;
        let bucket_size = range / 10.0; // 10 buckets
        let mut distribution = vec![0u32; 10];

        for &height in &terrain.heights {
            let bucket_idx = if range > 0.0 {
                ((height - min_height) / bucket_size).floor() as usize
            } else {
                0
            };
            let bucket_idx = bucket_idx.min(9); // Cap at last bucket
            distribution[bucket_idx] += 1;
        }

        let height_distribution = distribution
            .iter()
            .enumerate()
            .map(|(i, &count)| {
                let range_start = min_height + i as f32 * bucket_size;
                let range_end = range_start + bucket_size;
                (format!("{:.1}-{:.1}", range_start, range_end), count)
            })
            .collect();

        Self {
            dimensions: (terrain.width, terrain.height),
            scale: terrain.scale,
            world_size: (
                terrain.width as f32 * terrain.scale,
                terrain.height as f32 * terrain.scale,
            ),
            total_vertices: terrain.width * terrain.height,
            height_range: (min_height, max_height),
            mean_height: mean,
            std_deviation: std_dev,
            height_distribution,
        }
    }
}

impl SpawnZoneStats {
    fn analyze(zones: &[minion::map::SpawnZone]) -> Self {
        if zones.is_empty() {
            return Self {
                total_zones: 0,
                total_max_enemies: 0,
                average_radius: 0.0,
                radius_range: (0.0, 0.0),
                enemy_types: HashMap::new(),
                zone_distribution: Vec::new(),
            };
        }

        let total_max_enemies = zones.iter().map(|z| z.max_enemies).sum();
        let total_radius: f32 = zones.iter().map(|z| z.radius).sum();
        let average_radius = total_radius / zones.len() as f32;

        let min_radius = zones.iter().map(|z| z.radius).fold(f32::INFINITY, f32::min);
        let max_radius = zones
            .iter()
            .map(|z| z.radius)
            .fold(f32::NEG_INFINITY, f32::max);

        // Count enemy types
        let mut enemy_types = HashMap::new();
        for zone in zones {
            for enemy_type in &zone.enemy_types {
                *enemy_types.entry(enemy_type.clone()).or_insert(0) += 1;
            }
        }

        // Create radius distribution
        let radius_range = max_radius - min_radius;
        let bucket_size = if radius_range > 0.0 {
            radius_range / 5.0
        } else {
            1.0
        }; // 5 buckets
        let mut distribution = vec![0u32; 5];

        for zone in zones {
            let bucket_idx = if radius_range > 0.0 {
                ((zone.radius - min_radius) / bucket_size).floor() as usize
            } else {
                0
            };
            let bucket_idx = bucket_idx.min(4);
            distribution[bucket_idx] += 1;
        }

        let zone_distribution = distribution
            .iter()
            .enumerate()
            .map(|(i, &count)| {
                let range_start = min_radius + i as f32 * bucket_size;
                let range_end = range_start + bucket_size;
                (format!("{:.1}-{:.1}", range_start, range_end), count)
            })
            .collect();

        Self {
            total_zones: zones.len(),
            total_max_enemies,
            average_radius,
            radius_range: (min_radius, max_radius),
            enemy_types,
            zone_distribution,
        }
    }
}

impl ObjectStats {
    fn analyze(objects: &[minion::map::EnvironmentObject]) -> Self {
        if objects.is_empty() {
            return Self {
                total_objects: 0,
                object_types: HashMap::new(),
                position_bounds: ((0.0, 0.0, 0.0), (0.0, 0.0, 0.0)),
                scale_range: (0.0, 0.0),
                mean_scale: 0.0,
            };
        }

        // Count object types
        let mut object_types = HashMap::new();
        for obj in objects {
            *object_types.entry(obj.object_type.clone()).or_insert(0) += 1;
        }

        // Calculate position bounds
        let mut min_pos = Vec3::splat(f32::INFINITY);
        let mut max_pos = Vec3::splat(f32::NEG_INFINITY);

        for obj in objects {
            min_pos = min_pos.min(obj.position);
            max_pos = max_pos.max(obj.position);
        }

        // Calculate scale statistics (using X component as representative)
        let scales: Vec<f32> = objects.iter().map(|obj| obj.scale.x).collect();
        let min_scale = scales.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let max_scale = scales.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        let mean_scale = scales.iter().sum::<f32>() / scales.len() as f32;

        Self {
            total_objects: objects.len(),
            object_types,
            position_bounds: (
                (min_pos.x, min_pos.y, min_pos.z),
                (max_pos.x, max_pos.y, max_pos.z),
            ),
            scale_range: (min_scale, max_scale),
            mean_scale,
        }
    }
}

fn get_file_size(path: &Path) -> u64 {
    std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
}

fn print_text_format(stats: &MapStats, verbose: bool, section: &str) {
    println!("=== Map Information ===");
    println!("Name: {}", stats.name);
    println!(
        "File size: {} bytes ({:.1} KB)",
        stats.file_size_bytes,
        stats.file_size_bytes as f32 / 1024.0
    );
    println!();

    if section == "all" || section == "terrain" {
        println!("=== Terrain ===");
        println!(
            "Dimensions: {}x{} ({}x{} world units)",
            stats.terrain.dimensions.0,
            stats.terrain.dimensions.1,
            stats.terrain.world_size.0,
            stats.terrain.world_size.1
        );
        println!("Scale: {} units per grid cell", stats.terrain.scale);
        println!("Total vertices: {}", stats.terrain.total_vertices);

        // Calculate terrain bounds (same as terrain generation)
        let terrain_width = stats.terrain.world_size.0;
        let terrain_height = stats.terrain.world_size.1;
        let center_x_offset = terrain_width / 2.0;
        let center_z_offset = terrain_height / 2.0;
        println!(
            "World bounds: ({:.1}, {:.1}) to ({:.1}, {:.1})",
            -center_x_offset,
            -center_z_offset,
            center_x_offset - stats.terrain.scale,
            center_z_offset - stats.terrain.scale
        );

        println!(
            "Height range: {:.2} to {:.2} (span: {:.2})",
            stats.terrain.height_range.0,
            stats.terrain.height_range.1,
            stats.terrain.height_range.1 - stats.terrain.height_range.0
        );
        println!("Mean height: {:.2}", stats.terrain.mean_height);
        println!("Standard deviation: {:.2}", stats.terrain.std_deviation);

        if verbose {
            println!("Height distribution:");
            for (range, count) in &stats.terrain.height_distribution {
                let percentage = (*count as f32 / stats.terrain.total_vertices as f32) * 100.0;
                println!("  {}: {} vertices ({:.1}%)", range, count, percentage);
            }
        }
        println!();
    }

    if section == "all" || section == "spawns" {
        println!("=== Spawn Zones ===");
        println!("Total zones: {}", stats.spawn_zones.total_zones);
        println!("Total max enemies: {}", stats.spawn_zones.total_max_enemies);
        if stats.spawn_zones.total_zones > 0 {
            println!("Average radius: {:.2}", stats.spawn_zones.average_radius);
            println!(
                "Radius range: {:.2} to {:.2}",
                stats.spawn_zones.radius_range.0, stats.spawn_zones.radius_range.1
            );

            if verbose && !stats.spawn_zones.enemy_types.is_empty() {
                println!("Enemy types:");
                for (enemy_type, count) in &stats.spawn_zones.enemy_types {
                    println!("  {}: {} zones", enemy_type, count);
                }

                println!("Zone radius distribution:");
                for (range, count) in &stats.spawn_zones.zone_distribution {
                    println!("  {}: {} zones", range, count);
                }
            }
        }
        println!();
    }

    if section == "all" || section == "objects" {
        println!("=== Environment Objects ===");
        println!("Total objects: {}", stats.environment_objects.total_objects);
        if stats.environment_objects.total_objects > 0 {
            let bounds = &stats.environment_objects.position_bounds;
            println!(
                "Position bounds: ({:.1}, {:.1}, {:.1}) to ({:.1}, {:.1}, {:.1})",
                bounds.0.0, bounds.0.1, bounds.0.2, bounds.1.0, bounds.1.1, bounds.1.2
            );
            println!(
                "Scale range: {:.2} to {:.2} (mean: {:.2})",
                stats.environment_objects.scale_range.0,
                stats.environment_objects.scale_range.1,
                stats.environment_objects.mean_scale
            );

            if verbose {
                println!("Object types:");
                for (obj_type, count) in &stats.environment_objects.object_types {
                    let percentage =
                        (*count as f32 / stats.environment_objects.total_objects as f32) * 100.0;
                    println!("  {}: {} objects ({:.1}%)", obj_type, count, percentage);
                }
            }
        }
        println!();
    }
}

fn main() -> MinionResult<()> {
    let args = Args::parse();

    // Validate section argument
    let valid_sections = ["all", "terrain", "spawns", "objects"];
    if !valid_sections.contains(&args.section.as_str()) {
        return Err(MinionError::InvalidMapData {
            reason: format!(
                "Invalid section '{}'. Valid sections: {}",
                args.section,
                valid_sections.join(", ")
            ),
        });
    }

    // Validate format argument
    let valid_formats = ["text", "json"];
    if !valid_formats.contains(&args.format.as_str()) {
        return Err(MinionError::InvalidMapData {
            reason: format!(
                "Invalid format '{}'. Valid formats: {}",
                args.format,
                valid_formats.join(", ")
            ),
        });
    }

    // Load the map
    let map = MapDefinition::load_from_file(&args.input)?;

    // Get file size
    let maps_dir = MapDefinition::get_maps_dir()?;
    let file_path = maps_dir.join(&args.input);
    let file_size = get_file_size(&file_path);

    // Analyze the map
    let stats = MapStats {
        name: map.name.clone(),
        file_size_bytes: file_size,
        terrain: TerrainStats::analyze(&map.terrain),
        spawn_zones: SpawnZoneStats::analyze(&map.enemy_zones),
        environment_objects: ObjectStats::analyze(&map.environment_objects),
    };

    // Output results
    match args.format.as_str() {
        "text" => {
            print_text_format(&stats, args.verbose, &args.section);

            // Add player spawn information
            if args.section == "all" || args.section == "terrain" {
                println!("=== Player Spawn ===");
                println!(
                    "Spawn position: ({:.2}, {:.2}, {:.2})",
                    map.player_spawn.x, map.player_spawn.y, map.player_spawn.z
                );

                if let Some(terrain_height) =
                    map.get_height_at_world(map.player_spawn.x, map.player_spawn.z)
                {
                    println!("Terrain height at spawn: {:.2}", terrain_height);
                    println!(
                        "Adjusted spawn Y (terrain + 0.5): {:.2}",
                        terrain_height + 0.5
                    );
                    let height_diff = map.player_spawn.y - terrain_height;
                    if height_diff.abs() > 0.1 {
                        println!(
                            "WARNING: Player spawn Y ({:.2}) differs from terrain height ({:.2}) by {:.2}",
                            map.player_spawn.y, terrain_height, height_diff
                        );
                    } else {
                        println!("✓ Player spawn Y matches terrain height");
                    }
                } else {
                    println!(
                        "WARNING: Cannot sample terrain height at spawn position - may be out of bounds"
                    );
                }
                println!();
            }
        }
        "json" => {
            // Simple JSON output without external dependencies
            let json = format!(
                r#"{{
  "name": "{}",
  "file_size_bytes": {},
  "terrain": {{
    "dimensions": [{}, {}],
    "scale": {},
    "world_size": [{}, {}],
    "total_vertices": {},
    "height_range": [{}, {}],
    "mean_height": {},
    "std_deviation": {}
  }},
  "spawn_zones": {{
    "total_zones": {},
    "total_max_enemies": {},
    "average_radius": {},
    "radius_range": [{}, {}]
  }},
  "environment_objects": {{
    "total_objects": {},
    "scale_range": [{}, {}],
    "mean_scale": {}
  }}
}}"#,
                stats.name,
                stats.file_size_bytes,
                stats.terrain.dimensions.0,
                stats.terrain.dimensions.1,
                stats.terrain.scale,
                stats.terrain.world_size.0,
                stats.terrain.world_size.1,
                stats.terrain.total_vertices,
                stats.terrain.height_range.0,
                stats.terrain.height_range.1,
                stats.terrain.mean_height,
                stats.terrain.std_deviation,
                stats.spawn_zones.total_zones,
                stats.spawn_zones.total_max_enemies,
                stats.spawn_zones.average_radius,
                stats.spawn_zones.radius_range.0,
                stats.spawn_zones.radius_range.1,
                stats.environment_objects.total_objects,
                stats.environment_objects.scale_range.0,
                stats.environment_objects.scale_range.1,
                stats.environment_objects.mean_scale
            );

            if let Some(output_file) = &args.output {
                std::fs::write(output_file, json).map_err(|e| MinionError::InvalidMapData {
                    reason: format!("Failed to write output file: {}", e),
                })?;
                println!("Map information exported to: {}", output_file);
            } else {
                println!("{}", json);
            }
        }
        _ => unreachable!(),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use minion::map::{EnvironmentObject, SpawnZone, TerrainData};

    #[test]
    fn test_terrain_stats_analysis() {
        let terrain = TerrainData::new(2, 2, vec![0.0, 2.0, 4.0, 6.0], 1.0).unwrap();

        let stats = TerrainStats::analyze(&terrain);
        assert_eq!(stats.dimensions, (2, 2));
        assert_eq!(stats.total_vertices, 4);
        assert_eq!(stats.height_range, (0.0, 6.0));
        assert_eq!(stats.mean_height, 3.0);
    }

    #[test]
    fn test_empty_collections() {
        let spawn_stats = SpawnZoneStats::analyze(&[]);
        assert_eq!(spawn_stats.total_zones, 0);

        let object_stats = ObjectStats::analyze(&[]);
        assert_eq!(object_stats.total_objects, 0);
    }
}
