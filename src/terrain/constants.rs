/// Constants for terrain generation system
/// Default values for path generation
pub const DEFAULT_MAIN_ROADS: u32 = 3;
pub const DEFAULT_TRAILS_PER_BIOME: u32 = 2;
pub const DEFAULT_MIN_PATH_LENGTH: u32 = 50;
pub const DEFAULT_MAX_SLOPE_GRADIENT: f32 = 0.3;
pub const DEFAULT_PATH_WIDTH_MAIN_ROAD: f32 = 4.0;
pub const DEFAULT_PATH_WIDTH_TRAIL: f32 = 2.5;
pub const DEFAULT_PATH_WIDTH_MOUNTAIN_PASS: f32 = 2.0;
pub const DEFAULT_PATH_WIDTH_RIVER_PATH: f32 = 1.5;

/// Default values for biome generation
pub const DEFAULT_BIOME_REGIONS: u32 = 6;
pub const DEFAULT_TRANSITION_RADIUS: f32 = 30.0;
pub const MIN_VORONOI_SITES: usize = 3;

/// Pathfinding constants
pub const ASTAR_CARDINAL_COST: u32 = 10;
pub const ASTAR_DIAGONAL_COST: u32 = 14;
pub const MAX_PATH_ATTEMPTS: u32 = 1000;

/// Terrain sampling fallback values
pub const FALLBACK_TERRAIN_HEIGHT: f32 = 0.0;
pub const FALLBACK_BIOME_SUITABILITY: f32 = 0.1;

/// Object placement constants
pub const MAX_OBJECT_PLACEMENT_ATTEMPTS: u32 = 10;
pub const DEFAULT_OBJECT_DENSITY: f32 = 0.1;

/// Coordinate transformation constants
pub const GRID_INTERPOLATION_MARGIN: f32 = 1.0;
