//! Smooth Terrain Utility
//!
//! This utility applies smoothing filters to terrain heightmaps to reduce
//! sharp features and create more natural-looking landscapes.
//!
//! # Example Usage
//! ```bash
//! # Apply 3 passes of smoothing with default strength
//! cargo run --example smooth_terrain -- --input mountains.bin --output smooth_mountains.bin --passes 3
//!
//! # Light smoothing with custom strength
//! cargo run --example smooth_terrain -- --input hills.bin --output gentle_hills.bin --passes 1 --strength 0.3
//!
//! # Strong smoothing with many passes
//! cargo run --example smooth_terrain -- --input rough_terrain.bin --output flat_terrain.bin --passes 10 --strength 0.8
//!
//! # Dry run to see effects without making changes
//! cargo run --example smooth_terrain -- --input map.bin --output test.bin --passes 5 --dry-run
//! ```

use bevy::prelude::*;
use clap::Parser;
use minion::game_logic::errors::{MinionError, MinionResult};
use minion::map::{MapDefinition, TerrainData};

#[derive(Parser)]
#[command(name = "smooth_terrain")]
#[command(about = "Apply smoothing filters to terrain heightmaps")]
struct Args {
    /// Input map file (in maps/ directory)
    #[arg(long)]
    input: String,

    /// Output map file (in maps/ directory)
    #[arg(long)]
    output: String,

    /// Number of smoothing passes to apply
    #[arg(long, default_value = "1")]
    passes: u32,

    /// Smoothing strength (0.0-1.0, higher = more smoothing per pass)
    #[arg(long, default_value = "0.5")]
    strength: f32,

    /// Smoothing algorithm: box, gaussian
    #[arg(long, default_value = "box")]
    algorithm: String,

    /// Show what would be changed without making changes
    #[arg(long, default_value = "false")]
    dry_run: bool,

    /// Verbose output with statistics
    #[arg(short, long, default_value = "false")]
    verbose: bool,
}

#[derive(Debug, Clone)]
enum SmoothingAlgorithm {
    Box,
    Gaussian,
}

impl SmoothingAlgorithm {
    fn from_str(s: &str) -> MinionResult<Self> {
        match s.to_lowercase().as_str() {
            "box" => Ok(Self::Box),
            "gaussian" => Ok(Self::Gaussian),
            _ => Err(MinionError::InvalidMapData {
                reason: format!(
                    "Unknown smoothing algorithm '{}'. Available: box, gaussian",
                    s
                ),
            }),
        }
    }
}

fn calculate_terrain_statistics(terrain: &TerrainData) -> (f32, f32, f32, f32) {
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

    (min_height, max_height, mean, std_dev)
}

fn apply_box_smoothing(terrain: &TerrainData, strength: f32) -> MinionResult<TerrainData> {
    let width = terrain.width;
    let height = terrain.height;
    let mut new_heights = terrain.heights.clone();

    // Apply 3x3 box filter
    for y in 1..(height - 1) {
        for x in 1..(width - 1) {
            let center_idx = (y * width + x) as usize;
            let current_height = terrain.heights[center_idx];

            // Calculate average of 3x3 neighborhood
            let mut sum = 0.0;
            let mut count = 0;

            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;

                    if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                        let idx = (ny as u32 * width + nx as u32) as usize;
                        sum += terrain.heights[idx];
                        count += 1;
                    }
                }
            }

            let average = sum / count as f32;

            // Blend between original and smoothed height based on strength
            new_heights[center_idx] = current_height * (1.0 - strength) + average * strength;
        }
    }

    TerrainData::new(width, height, new_heights, terrain.scale)
}

fn apply_gaussian_smoothing(terrain: &TerrainData, strength: f32) -> MinionResult<TerrainData> {
    let width = terrain.width;
    let height = terrain.height;
    let mut new_heights = terrain.heights.clone();

    // Gaussian kernel weights (3x3)
    let kernel = [
        [0.0625, 0.125, 0.0625], // 1/16, 2/16, 1/16
        [0.125, 0.25, 0.125],    // 2/16, 4/16, 2/16
        [0.0625, 0.125, 0.0625], // 1/16, 2/16, 1/16
    ];

    // Apply Gaussian filter
    for y in 1..(height - 1) {
        for x in 1..(width - 1) {
            let center_idx = (y * width + x) as usize;
            let current_height = terrain.heights[center_idx];

            // Apply Gaussian kernel
            let mut weighted_sum = 0.0;

            for (dy, kernel_row) in kernel.iter().enumerate() {
                for (dx, &weight) in kernel_row.iter().enumerate() {
                    let nx = x + dx as u32 - 1;
                    let ny = y + dy as u32 - 1;
                    let idx = (ny * width + nx) as usize;
                    weighted_sum += terrain.heights[idx] * weight;
                }
            }

            // Blend between original and smoothed height based on strength
            new_heights[center_idx] = current_height * (1.0 - strength) + weighted_sum * strength;
        }
    }

    TerrainData::new(width, height, new_heights, terrain.scale)
}

fn main() -> MinionResult<()> {
    let args = Args::parse();

    // Validate arguments
    if args.passes == 0 {
        return Err(MinionError::InvalidMapData {
            reason: "Number of passes must be greater than 0".to_string(),
        });
    }

    if !(0.0..=1.0).contains(&args.strength) {
        return Err(MinionError::InvalidMapData {
            reason: "Strength must be between 0.0 and 1.0".to_string(),
        });
    }

    let algorithm = SmoothingAlgorithm::from_str(&args.algorithm)?;

    // Load the input map
    let mut map = MapDefinition::load_from_file(&args.input)?;

    if args.verbose {
        println!(
            "Loaded map '{}' with terrain {}x{}",
            map.name, map.terrain.width, map.terrain.height
        );

        let (min, max, mean, std_dev) = calculate_terrain_statistics(&map.terrain);
        println!("Original terrain statistics:");
        println!(
            "  Height range: {:.2} to {:.2} (span: {:.2})",
            min,
            max,
            max - min
        );
        println!("  Mean height: {:.2}", mean);
        println!("  Standard deviation: {:.2}", std_dev);
        println!();
        println!(
            "Applying {} algorithm with {} passes at {:.1}% strength",
            args.algorithm,
            args.passes,
            args.strength * 100.0
        );
    }

    // Apply smoothing passes
    let mut smoothed_terrain = map.terrain.clone();

    for pass in 1..=args.passes {
        if args.verbose {
            println!("Applying smoothing pass {}...", pass);
        }

        smoothed_terrain = match algorithm {
            SmoothingAlgorithm::Box => apply_box_smoothing(&smoothed_terrain, args.strength)?,
            SmoothingAlgorithm::Gaussian => {
                apply_gaussian_smoothing(&smoothed_terrain, args.strength)?
            }
        };
    }

    // Calculate and display results
    if args.verbose {
        let (new_min, new_max, new_mean, new_std_dev) =
            calculate_terrain_statistics(&smoothed_terrain);
        println!();
        println!("Smoothed terrain statistics:");
        println!(
            "  Height range: {:.2} to {:.2} (span: {:.2})",
            new_min,
            new_max,
            new_max - new_min
        );
        println!("  Mean height: {:.2}", new_mean);
        println!("  Standard deviation: {:.2}", new_std_dev);

        let (orig_min, orig_max, _orig_mean, orig_std_dev) =
            calculate_terrain_statistics(&map.terrain);
        println!();
        println!("Changes:");
        println!(
            "  Height span: {:.2} -> {:.2} ({:.1}% change)",
            orig_max - orig_min,
            new_max - new_min,
            ((new_max - new_min) / (orig_max - orig_min) - 1.0) * 100.0
        );
        println!(
            "  Standard deviation: {:.2} -> {:.2} ({:.1}% change)",
            orig_std_dev,
            new_std_dev,
            (new_std_dev / orig_std_dev - 1.0) * 100.0
        );
    }

    println!(
        "Smoothing completed: {} passes with {:.1}% strength",
        args.passes,
        args.strength * 100.0
    );

    if args.dry_run {
        println!("Dry run - no changes made to files");
        return Ok(());
    }

    // Update map with smoothed terrain
    map.terrain = smoothed_terrain;

    // Save the smoothed map
    map.save_to_file(&args.output)?;

    println!("Smoothed map saved to: {}", args.output);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smoothing_algorithms() {
        let terrain = TerrainData::new(
            3,
            3,
            vec![0.0, 5.0, 0.0, 5.0, 10.0, 5.0, 0.0, 5.0, 0.0],
            1.0,
        )
        .unwrap();

        // Test box smoothing
        let smoothed = apply_box_smoothing(&terrain, 1.0).unwrap();

        // Center should be close to average of all values
        let center_idx = 1 * 3 + 1; // center of 3x3
        assert!(smoothed.heights[center_idx] < terrain.heights[center_idx]);

        // Test Gaussian smoothing
        let gaussian_smoothed = apply_gaussian_smoothing(&terrain, 1.0).unwrap();
        assert!(gaussian_smoothed.heights[center_idx] < terrain.heights[center_idx]);
    }

    #[test]
    fn test_terrain_statistics() {
        let terrain = TerrainData::new(2, 2, vec![0.0, 2.0, 4.0, 6.0], 1.0).unwrap();

        let (min, max, mean, _std_dev) = calculate_terrain_statistics(&terrain);
        assert_eq!(min, 0.0);
        assert_eq!(max, 6.0);
        assert_eq!(mean, 3.0); // (0+2+4+6)/4 = 3
    }

    #[test]
    fn test_smoothing_algorithm_parsing() {
        assert!(matches!(
            SmoothingAlgorithm::from_str("box"),
            Ok(SmoothingAlgorithm::Box)
        ));
        assert!(matches!(
            SmoothingAlgorithm::from_str("gaussian"),
            Ok(SmoothingAlgorithm::Gaussian)
        ));
        assert!(SmoothingAlgorithm::from_str("invalid").is_err());
    }
}
