use clap::Parser;
use minion::game_logic::errors::MinionResult;
use minion::map::MapDefinition;

mod mapgen {
    pub mod cli_utils;
    pub mod map_generator;
    pub mod terrain_builder;
}

use mapgen::cli_utils::*;
use mapgen::map_generator::{MapGenerationConfig, MapGenerator};
use mapgen::terrain_builder::TerrainBuilder;

#[derive(Parser, Clone)]
#[command(name = "mapgen")]
#[command(about = "Generate basic map files for the Minion ARPG")]
struct Args {
    /// Map name
    #[arg(long, default_value = "generated_map")]
    name: String,

    /// Terrain size in grid cells (format: WIDTHxHEIGHT)
    #[arg(long, default_value = "64x64")]
    size: String,

    /// Output file path relative to assets/maps/ directory (e.g., "my_map.bin" or "folder/my_map.bin")
    #[arg(long)]
    output: Option<String>,

    /// Player spawn position (format: X,Y,Z)
    #[arg(long, default_value = "0.0,1.0,0.0")]
    player_spawn: String,

    /// Terrain type preset (flat, hills, mountains, valleys)
    #[arg(long, default_value = "flat")]
    terrain_type: String,

    /// Random seed for reproducible generation
    #[arg(long)]
    seed: Option<u32>,

    /// Terrain amplitude (height variation)
    #[arg(long, default_value = "10.0")]
    amplitude: f32,

    /// Base frequency for noise (terrain feature density)
    #[arg(long, default_value = "0.01")]
    frequency: f32,

    /// Number of noise octaves for detail
    #[arg(long, default_value = "4")]
    octaves: u32,

    /// Object density (0.0-1.0, higher = more objects)
    #[arg(long, default_value = "0.1")]
    objects: f32,

    /// Comma-separated list of object types to place
    #[arg(long, default_value = "tree,rock")]
    object_types: String,

    /// Object scale range as min,max (e.g., "0.8,1.2")
    #[arg(long, default_value = "0.8,1.2")]
    object_scale: String,

    /// Terrain scale (world units per grid cell - smaller = higher density)
    #[arg(long, default_value = "0.5")]
    scale: f32,

    /// Enable biome generation for varied terrain types
    #[arg(long)]
    biomes: bool,

    /// Number of biome regions (only used with --biomes)
    #[arg(long, default_value = "6")]
    biome_regions: u32,

    /// Enable path network generation
    #[arg(long)]
    paths: bool,

    /// Number of main roads to generate (only used with --paths)
    #[arg(long, default_value = "3")]
    main_roads: u32,

    /// Number of trails per biome region (only used with --paths)
    #[arg(long, default_value = "2")]
    trails_per_biome: u32,
}

fn validate_output_path(filename: &str) -> MinionResult<()> {
    use std::path::Path;

    // Check for absolute paths which would be problematic
    let path = Path::new(filename);
    if path.is_absolute() {
        return Err(minion::game_logic::errors::MinionError::InvalidMapData {
            reason: format!(
                "Output path must be relative to assets/maps/ directory, got absolute path: {}",
                filename
            ),
        });
    }

    // Check for parent directory traversal attempts
    if filename.contains("..") {
        return Err(minion::game_logic::errors::MinionError::InvalidMapData {
            reason: "Output path cannot contain '..' for security reasons".to_string(),
        });
    }

    Ok(())
}

fn main() -> MinionResult<()> {
    let args = Args::parse();

    // Parse and validate all CLI arguments
    let (width, height) = parse_size(&args.size)?;
    let player_spawn = parse_position(&args.player_spawn)?;
    let object_types = parse_object_types(&args.object_types);
    let scale_range = parse_scale_range(&args.object_scale)?;
    let object_density = validate_density(args.objects);
    let output_filename = args.output.unwrap_or_else(|| format!("{}.bin", args.name));

    // Validate output path early to catch obvious issues
    validate_output_path(&output_filename)?;

    // Build terrain generator using builder pattern
    let generator = TerrainBuilder::new(args.terrain_type)
        .seed(args.seed)
        .amplitude(args.amplitude)
        .frequency(args.frequency)
        .octaves(args.octaves)
        .build()?;

    // Create map generation config
    let config = MapGenerationConfig {
        name: args.name.clone(),
        width,
        height,
        player_spawn,
        generator,
        object_density,
        object_types,
        scale_range,
        terrain_scale: args.scale,
        enable_biomes: args.biomes,
        biome_regions: args.biome_regions,
        enable_paths: args.paths,
        main_roads: args.main_roads,
        trails_per_biome: args.trails_per_biome,
    };

    // Generate the map
    let map = MapGenerator::generate(config)?;

    // Save and display results
    map.save_to_file(&output_filename)?;

    print_map_summary(&map, &output_filename)
}

fn print_map_summary(map: &MapDefinition, output_filename: &str) -> MinionResult<()> {
    let maps_dir = MapDefinition::get_maps_dir()?;
    let full_path = maps_dir.join(output_filename);

    println!("Map saved successfully to: {}", full_path.display());
    println!("\nMap summary:");
    println!("  Name: {}", map.name);
    println!(
        "  Terrain: {}x{} at scale {} (total {} height points)",
        map.terrain.width,
        map.terrain.height,
        map.terrain.scale,
        map.terrain.heights.len()
    );
    println!("  Player spawn: {}", map.player_spawn);
    println!(
        "  Enemy zones: {} zones with {} total enemy types",
        map.enemy_zones.len(),
        map.enemy_zones
            .iter()
            .map(|z| z.enemy_types.len())
            .sum::<usize>()
    );
    println!(
        "  Environment objects: {} objects",
        map.environment_objects.len()
    );

    for (i, zone) in map.enemy_zones.iter().enumerate() {
        println!(
            "    Zone {}: center={}, radius={}, max_enemies={}, types={:?}",
            i + 1,
            zone.center,
            zone.radius,
            zone.max_enemies,
            zone.enemy_types
        );
    }

    if !map.environment_objects.is_empty() {
        let mut type_counts = std::collections::HashMap::new();
        for obj in &map.environment_objects {
            *type_counts.entry(&obj.object_type).or_insert(0) += 1;
        }
        println!("  Object types:");
        for (obj_type, count) in type_counts {
            println!("    {obj_type}: {count} objects");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_integration() {
        // Integration test to ensure modules work together
        let args = Args {
            name: "test".to_string(),
            size: "32x32".to_string(),
            output: Some("test_output.bin".to_string()),
            player_spawn: "16.0,1.0,16.0".to_string(),
            terrain_type: "flat".to_string(),
            seed: Some(12345),
            amplitude: 10.0,
            frequency: 0.1,
            octaves: 4,
            objects: 0.1,
            object_types: "tree,rock".to_string(),
            object_scale: "0.8,1.2".to_string(),
            scale: 0.5,
            biomes: false,
            biome_regions: 6,
            paths: false,
            main_roads: 3,
            trails_per_biome: 2,
        };

        // Test parsing
        let (width, height) = parse_size(&args.size).unwrap();
        assert_eq!((width, height), (32, 32));

        let player_spawn = parse_position(&args.player_spawn).unwrap();
        assert_eq!(player_spawn, bevy::prelude::Vec3::new(16.0, 1.0, 16.0));

        // Test terrain builder
        let generator = TerrainBuilder::new(args.terrain_type.clone())
            .seed(args.seed)
            .build()
            .unwrap();
        assert_eq!(generator.seed, 12345);
    }
}
