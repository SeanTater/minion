//! Add Objects Utility
//!
//! This utility adds environment objects to existing maps, with terrain-aware
//! placement and collision avoidance.
//!
//! # Example Usage
//! ```bash
//! # Add 50 trees to a map with default density
//! cargo run --example add_objects -- --input map.bin --output map_with_trees.bin --type tree --count 50
//!
//! # Add random objects with high density
//! cargo run --example add_objects -- --input hills.bin --output dense_hills.bin --density 0.8 --count 100
//!
//! # Add specific object types with custom scale range
//! cargo run --example add_objects -- --input map.bin --output custom_map.bin --types "tree,rock,bush" --count 75 --scale "0.5,2.0"
//!
//! # Dry run to see placement without making changes  
//! cargo run --example add_objects -- --input map.bin --output test.bin --type rock --count 20 --dry-run
//! ```

use bevy::prelude::*;
use clap::Parser;
use minion::game_logic::errors::{MinionError, MinionResult};
use minion::map::{EnvironmentObject, MapDefinition};
use minion::terrain_generation::is_suitable_for_spawning;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;

#[derive(Parser)]
#[command(name = "add_objects")]
#[command(about = "Add environment objects to existing maps")]
struct Args {
    /// Input map file (in maps/ directory)
    #[arg(long)]
    input: String,

    /// Output map file (in maps/ directory)
    #[arg(long)]
    output: String,

    /// Specific object type to add (overrides random selection)
    #[arg(long)]
    r#type: Option<String>,

    /// Comma-separated list of object types to randomly choose from
    #[arg(long, default_value = "tree,rock,bush")]
    types: String,

    /// Number of objects to attempt to place
    #[arg(long, default_value = "50")]
    count: u32,

    /// Object density factor (0.0-1.0, affects placement success rate)
    #[arg(long, default_value = "0.1")]
    density: f32,

    /// Object scale range as "min,max" (e.g., "0.8,1.2")
    #[arg(long, default_value = "0.8,1.2")]
    scale: String,

    /// Minimum distance between objects
    #[arg(long, default_value = "2.0")]
    min_distance: f32,

    /// Random seed for reproducible placement
    #[arg(long)]
    seed: Option<u32>,

    /// Show what would be added without making changes
    #[arg(long, default_value = "false")]
    dry_run: bool,

    /// Verbose output
    #[arg(short, long, default_value = "false")]
    verbose: bool,
}

fn parse_scale_range(scale_str: &str) -> MinionResult<(f32, f32)> {
    let parts: Vec<&str> = scale_str.split(',').collect();
    if parts.len() != 2 {
        return Err(MinionError::InvalidMapData {
            reason: format!(
                "Invalid scale format '{scale_str}'. Expected format: min,max (e.g., 0.8,1.2)"
            ),
        });
    }

    let min = parts[0]
        .parse::<f32>()
        .map_err(|_| MinionError::InvalidMapData {
            reason: format!("Invalid minimum scale value: '{}'", parts[0]),
        })?;

    let max = parts[1]
        .parse::<f32>()
        .map_err(|_| MinionError::InvalidMapData {
            reason: format!("Invalid maximum scale value: '{}'", parts[1]),
        })?;

    if min <= 0.0 || max <= 0.0 || min > max {
        return Err(MinionError::InvalidMapData {
            reason: "Scale values must be positive and min <= max".to_string(),
        });
    }

    Ok((min, max))
}

fn parse_object_types(types_str: &str) -> Vec<String> {
    types_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn is_position_valid(
    position: Vec3,
    existing_objects: &[EnvironmentObject],
    min_distance: f32,
) -> bool {
    for obj in existing_objects {
        if position.distance(obj.position) < min_distance {
            return false;
        }
    }
    true
}

fn main() -> MinionResult<()> {
    let args = Args::parse();

    // Validate arguments
    if args.count == 0 {
        return Err(MinionError::InvalidMapData {
            reason: "Count must be greater than 0".to_string(),
        });
    }

    if args.density < 0.0 || args.density > 1.0 {
        return Err(MinionError::InvalidMapData {
            reason: "Density must be between 0.0 and 1.0".to_string(),
        });
    }

    if args.min_distance < 0.0 {
        return Err(MinionError::InvalidMapData {
            reason: "Minimum distance must be non-negative".to_string(),
        });
    }

    let (min_scale, max_scale) = parse_scale_range(&args.scale)?;

    // Load the input map
    let mut map = MapDefinition::load_from_file(&args.input)?;
    let original_count = map.environment_objects.len();

    if args.verbose {
        println!(
            "Loaded map '{}' with {} existing objects",
            map.name, original_count
        );
    }

    // Setup object types
    let object_types = if let Some(specific_type) = &args.r#type {
        vec![specific_type.clone()]
    } else {
        parse_object_types(&args.types)
    };

    if object_types.is_empty() {
        return Err(MinionError::InvalidMapData {
            reason: "No object types specified".to_string(),
        });
    }

    if args.verbose {
        println!("Object types to place: {:?}", object_types);
        println!("Scale range: {:.2} - {:.2}", min_scale, max_scale);
        println!("Attempting to place {} objects", args.count);
    }

    // Setup RNG
    let seed = args.seed.unwrap_or_else(|| rand::random());
    let mut rng = Pcg64::seed_from_u64(seed as u64);

    if args.verbose {
        println!("Using seed: {}", seed);
    }

    // Calculate terrain bounds
    let terrain_width = map.terrain.width as f32 * map.terrain.scale;
    let terrain_height = map.terrain.height as f32 * map.terrain.scale;

    // Place objects
    let mut new_objects = Vec::new();
    let mut attempts = 0;
    let max_attempts = args.count * 10; // Prevent infinite loops

    while new_objects.len() < args.count as usize && attempts < max_attempts {
        attempts += 1;

        // Generate random position
        let x = rng.gen_range(0.0..terrain_width);
        let z = rng.gen_range(0.0..terrain_height);

        // Get terrain height at this position
        let y = map.get_height_at_world(x, z).unwrap_or(0.0);
        let position = Vec3::new(x, y, z);

        // Check if position is suitable (not too steep, etc.)
        if !is_suitable_for_spawning(&map.terrain, x, z, 0.5) {
            continue;
        }

        // Check density (probability of placement)
        if rng.gen_range(0.0..1.0) > args.density {
            continue;
        }

        // Check distance from existing objects
        if !is_position_valid(position, &map.environment_objects, args.min_distance) {
            continue;
        }

        // Check distance from new objects
        if !is_position_valid(position, &new_objects, args.min_distance) {
            continue;
        }

        // Choose object type
        let object_type = object_types[rng.gen_range(0..object_types.len())].clone();

        // Generate random scale
        let scale_factor = rng.gen_range(min_scale..=max_scale);
        let scale = Vec3::splat(scale_factor);

        // Generate random rotation (only Y-axis for most objects)
        let rotation = Vec3::new(0.0, rng.gen_range(0.0..std::f32::consts::TAU), 0.0);

        // Create the object
        let obj = EnvironmentObject::new(object_type, position, rotation, scale);
        new_objects.push(obj);

        if args.verbose && new_objects.len() % 10 == 0 {
            println!("Placed {} objects so far...", new_objects.len());
        }
    }

    let placed_count = new_objects.len();
    let success_rate = if attempts > 0 {
        (placed_count as f32 / attempts as f32) * 100.0
    } else {
        0.0
    };

    // Display results
    println!(
        "Object placement results: {} objects placed out of {} requested ({:.1}% success)",
        placed_count,
        args.count,
        if args.count > 0 {
            (placed_count as f32 / args.count as f32) * 100.0
        } else {
            0.0
        }
    );
    println!(
        "Placement attempts: {} (success rate: {:.1}%)",
        attempts, success_rate
    );
    println!(
        "Final object count: {} (was {})",
        original_count + placed_count,
        original_count
    );

    if args.verbose {
        for (i, obj) in new_objects.iter().enumerate() {
            println!(
                "  {}: {} at {:?} scale {:.2}",
                i + 1,
                obj.object_type,
                obj.position,
                obj.scale.x
            );
        }
    }

    if args.dry_run {
        println!("Dry run - no changes made to files");
        return Ok(());
    }

    // Add new objects to map
    map.environment_objects.extend(new_objects);

    // Save the updated map
    map.save_to_file(&args.output)?;

    println!("Updated map saved to: {}", args.output);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_scale_range() {
        let (min, max) = parse_scale_range("0.5,2.0").unwrap();
        assert_eq!(min, 0.5);
        assert_eq!(max, 2.0);

        // Test invalid formats
        assert!(parse_scale_range("invalid").is_err());
        assert!(parse_scale_range("1.0,0.5").is_err()); // min > max
        assert!(parse_scale_range("-1.0,2.0").is_err()); // negative
    }

    #[test]
    fn test_parse_object_types() {
        let types = parse_object_types("tree,rock,bush");
        assert_eq!(types, vec!["tree", "rock", "bush"]);

        let types = parse_object_types(" tree , rock , bush ");
        assert_eq!(types, vec!["tree", "rock", "bush"]);

        let types = parse_object_types("");
        assert!(types.is_empty());
    }

    #[test]
    fn test_position_validation() {
        let objects = vec![
            EnvironmentObject::simple("tree".to_string(), Vec3::new(0.0, 0.0, 0.0)),
            EnvironmentObject::simple("rock".to_string(), Vec3::new(5.0, 0.0, 0.0)),
        ];

        // Too close to first object
        assert!(!is_position_valid(Vec3::new(1.0, 0.0, 0.0), &objects, 2.0));

        // Far enough from all objects
        assert!(is_position_valid(Vec3::new(10.0, 0.0, 10.0), &objects, 2.0));
    }
}
