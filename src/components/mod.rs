use crate::resources::GameSettings;
use bevy::prelude::*;
use derive_more::{Add, Display, From, Mul};
use std::ops::Sub;

// Generic resource pool for health, mana, energy, etc.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Component)]
pub struct ResourcePool<T> {
    pub current: f32,
    pub max: f32,
    _marker: std::marker::PhantomData<T>,
}

// Resource type markers
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Health;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Mana;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Energy;

// Type aliases for convenience
pub type HealthPool = ResourcePool<Health>;
pub type ManaPool = ResourcePool<Mana>;
pub type EnergyPool = ResourcePool<Energy>;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Mul, Display, From)]
pub struct Speed(pub f32);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Add, Mul, Display, From)]
pub struct Distance(pub f32);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Add, Mul, Display, From)]
pub struct Damage(pub f32);

impl<T> ResourcePool<T> {
    pub fn new(current: f32, max: f32) -> Self {
        Self {
            current: current.max(0.0).min(max),
            max: max.max(0.0),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn new_full(max: f32) -> Self {
        Self::new(max, max)
    }

    pub fn is_empty(self) -> bool {
        self.current <= 0.0
    }
    pub fn is_full(self) -> bool {
        self.current >= self.max
    }

    pub fn percentage(self) -> f32 {
        if self.max > 0.0 {
            self.current / self.max
        } else {
            0.0
        }
    }

    pub fn restore(&mut self, amount: f32) {
        self.current = (self.current + amount).min(self.max);
    }

    pub fn consume(&mut self, amount: f32) -> bool {
        if self.current >= amount {
            self.current = (self.current - amount).max(0.0);
            true
        } else {
            false
        }
    }

    pub fn drain(&mut self, amount: f32) {
        self.current = (self.current - amount).max(0.0);
    }
}

// Health-specific methods
impl ResourcePool<Health> {
    pub fn is_dead(self) -> bool {
        self.current <= 0.0
    }

    pub fn take_damage(&mut self, damage: Damage) {
        self.current = (self.current - damage.0).max(0.0);
    }

    pub fn heal(&mut self, amount: f32) {
        self.restore(amount);
    }
}

// Mana-specific methods
impl ResourcePool<Mana> {
    pub fn spend(&mut self, cost: f32) -> bool {
        self.consume(cost)
    }
}

// Energy-specific methods
impl ResourcePool<Energy> {
    pub fn spend(&mut self, cost: f32) -> bool {
        self.consume(cost)
    }

    pub fn deplete(&mut self, amount: f32) {
        self.drain(amount);
    }
}

impl<T> std::fmt::Display for ResourcePool<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.0}/{:.0}", self.current, self.max)
    }
}

// Trait removed - using direct field access instead

impl Speed {
    pub fn new(value: f32) -> Self {
        Self(value.max(0.0))
    }
    pub const ZERO: Speed = Speed(0.0);
}

impl Distance {
    pub fn new(value: f32) -> Self {
        Self(value.max(0.0))
    }
    pub const ZERO: Distance = Distance(0.0);
}

// Manual implementations for operations not available in derive_more

impl Sub for Distance {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self((self.0 - rhs.0).max(0.0))
    }
}

impl Sub for Damage {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self((self.0 - rhs.0).max(0.0))
    }
}

impl std::ops::Div<f32> for Distance {
    type Output = Self;
    fn div(self, rhs: f32) -> Self::Output {
        Self(self.0 / rhs)
    }
}

impl Damage {
    pub fn new(value: f32) -> Self {
        Self(value.max(0.0))
    }
    pub const ZERO: Damage = Damage(0.0);
}

// Custom math operations for Vec3 * Speed
impl std::ops::Mul<Speed> for Vec3 {
    type Output = Vec3;
    fn mul(self, rhs: Speed) -> Self::Output {
        self * rhs.0
    }
}

// Custom math operations for f32 comparisons
impl PartialOrd<f32> for Distance {
    fn partial_cmp(&self, other: &f32) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl PartialEq<f32> for Distance {
    fn eq(&self, other: &f32) -> bool {
        self.0 == *other
    }
}

#[derive(Component)]
pub struct Player {
    pub move_target: Option<Vec3>,
    pub speed: Speed,
    pub health: HealthPool,
    pub mana: ManaPool,
    pub energy: EnergyPool,
}

#[derive(Component)]
pub struct Ground;

#[derive(Component)]
pub struct SceneLight;

#[derive(Component)]
pub struct CameraFollow {
    pub offset: Vec3,
}

#[derive(Component)]
pub struct Enemy {
    pub speed: Speed,
    pub health: HealthPool,
    pub mana: ManaPool,
    pub energy: EnergyPool,
    pub chase_distance: Distance,
    pub is_dying: bool,
}

#[derive(Component)]
pub struct Name(pub String);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LodLevel {
    High,   // Original high-poly model
    Medium, // Medium-poly model
    Low,    // Low-poly model
}

impl LodLevel {
    /// Parse LOD level from config string, defaulting to High for invalid values
    pub fn from_config_string(config_str: &str) -> Self {
        match config_str {
            "medium" => LodLevel::Medium,
            "low" => LodLevel::Low,
            _ => LodLevel::High, // Default to high if invalid string
        }
    }

    /// Apply global max LOD level cap to a desired LOD level
    pub fn apply_max_cap(desired: LodLevel, max_cap: LodLevel) -> LodLevel {
        match (desired, max_cap) {
            (LodLevel::High, LodLevel::Medium) | (LodLevel::High, LodLevel::Low) => max_cap,
            (LodLevel::Medium, LodLevel::Low) => LodLevel::Low,
            _ => desired,
        }
    }
}

#[derive(Component)]
pub struct LodEntity {
    pub current_level: LodLevel,
    pub high_handle: Handle<Scene>,
    pub med_handle: Handle<Scene>,
    pub low_handle: Handle<Scene>,
    pub entity_type: LodEntityType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LodEntityType {
    Player,
    Enemy,
}

// Unified resource display system
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResourceType {
    Health,
    Mana,
    Energy,
}

impl ResourceType {
    pub fn color(self) -> Color {
        match self {
            ResourceType::Health => Color::srgb(0.8, 0.2, 0.2),
            ResourceType::Mana => Color::srgb(0.2, 0.2, 0.8),
            ResourceType::Energy => Color::srgb(0.8, 0.8, 0.2),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ResourceType::Health => "HP",
            ResourceType::Mana => "MP",
            ResourceType::Energy => "EN",
        }
    }
}

#[derive(Component)]
pub struct ResourceDisplay {
    pub resource_type: ResourceType,
    pub target_entity: Entity,
    pub show_text: bool,
}

impl ResourceDisplay {
    pub fn new(resource_type: ResourceType, target_entity: Entity, show_text: bool) -> Self {
        Self {
            resource_type,
            target_entity,
            show_text,
        }
    }
}

#[derive(Component)]
pub struct Bullet {
    pub direction: Vec3,
    pub speed: Speed,
    pub lifetime: f32,
    pub damage: Damage,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AreaEffectType {
    Magic,
    Poison,
}

impl AreaEffectType {
    pub fn damage_per_second(&self, settings: &GameSettings) -> Damage {
        match self {
            AreaEffectType::Magic => Damage::new(settings.magic_damage_per_second),
            AreaEffectType::Poison => Damage::new(settings.poison_damage_per_second),
        }
    }

    pub fn radius(&self, settings: &GameSettings) -> Distance {
        match self {
            AreaEffectType::Magic => Distance::new(settings.magic_area_radius),
            AreaEffectType::Poison => Distance::new(settings.poison_area_radius),
        }
    }

    pub fn duration(&self, settings: &GameSettings) -> f32 {
        match self {
            AreaEffectType::Magic => settings.magic_area_duration,
            AreaEffectType::Poison => settings.poison_area_duration,
        }
    }

    pub fn base_color(&self) -> Color {
        match self {
            AreaEffectType::Magic => Color::srgba(0.0, 0.5, 1.0, 0.3),
            AreaEffectType::Poison => Color::srgba(0.0, 1.0, 0.2, 0.3),
        }
    }
}

#[derive(Component)]
pub struct AreaEffect {
    pub effect_type: AreaEffectType,
    pub elapsed: f32,
}

/// Pathfinding agent component that can be used by both players and enemies
#[derive(Component)]
pub struct PathfindingAgent {
    /// Current path as a series of waypoints in world coordinates
    pub current_path: Vec<Vec3>,
    /// Index of the next waypoint to reach in the current path
    pub path_index: usize,
    /// Final destination for this agent
    pub destination: Option<Vec3>,
    /// Time when the path was last recalculated
    pub last_replan_time: f32,
    /// Minimum time between path recalculations (seconds)
    pub replan_interval: f32,
    /// Distance threshold to consider a waypoint reached
    pub waypoint_reach_distance: f32,
    /// Maximum distance the agent can travel before replanning
    pub max_path_distance: f32,
    /// Agent physical radius used for planning (per-agent)
    pub agent_radius: f32,
}

impl PathfindingAgent {
    /// Create a new pathfinding agent with default settings
    pub fn new() -> Self {
        Self {
            current_path: Vec::new(),
            path_index: 0,
            destination: None,
            last_replan_time: 0.0,
            replan_interval: 0.5,         // Replan every 0.5 seconds
            waypoint_reach_distance: 1.0, // 1.0 units - works better with spaced waypoints
            max_path_distance: 50.0,      // Replan if destination changes by more than 50 units
            agent_radius: 0.5,            // Default per-agent radius
        }
    }
}

impl Default for PathfindingAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl PathfindingAgent {
    /// Get the current waypoint this agent is moving towards
    pub fn current_waypoint(&self) -> Option<Vec3> {
        self.current_path.get(self.path_index).copied()
    }

    /// Check if the agent has a valid path to follow
    pub fn has_path(&self) -> bool {
        !self.current_path.is_empty() && self.path_index < self.current_path.len()
    }

    /// Clear the current path
    pub fn clear_path(&mut self) {
        self.current_path.clear();
        self.path_index = 0;
    }

    /// Set a new path for the agent
    pub fn set_path(&mut self, path: Vec<Vec3>) {
        self.current_path = path;
        self.path_index = 0;
    }

    /// Advance to the next waypoint in the path
    pub fn advance_waypoint(&mut self) {
        if self.path_index < self.current_path.len() {
            self.path_index += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_pool_damage_system() {
        let mut health = HealthPool::new_full(100.0);
        let damage = Damage::new(30.0);

        health.take_damage(damage);
        assert_eq!(health.current, 70.0);
        assert!(!health.is_dead());

        let fatal_damage = Damage::new(100.0);
        health.take_damage(fatal_damage);
        assert_eq!(health.current, 0.0);
        assert!(health.is_dead());
    }

    #[test]
    fn test_health_pool_saturating_sub() {
        let mut health = HealthPool::new_full(10.0);
        let massive_damage = Damage::new(50.0);

        health.take_damage(massive_damage);
        assert_eq!(health.current, 0.0);
        assert!(health.is_dead());
    }

    #[test]
    fn test_health_pool_healing() {
        let mut health = HealthPool::new(50.0, 100.0);

        health.heal(30.0);
        assert_eq!(health.current, 80.0);
        assert_eq!(health.percentage(), 0.8);

        // Test healing beyond max
        health.heal(50.0);
        assert_eq!(health.current, 100.0);
        assert!(health.is_full());
    }

    #[test]
    fn test_mana_pool_spend_and_restore() {
        let mut mana = ManaPool::new_full(100.0);

        // Successful spend
        assert!(mana.spend(30.0));
        assert_eq!(mana.current, 70.0);
        assert_eq!(mana.percentage(), 0.7);

        // Failed spend (insufficient mana)
        assert!(!mana.spend(80.0));
        assert_eq!(mana.current, 70.0);

        // Restore mana
        mana.restore(40.0);
        assert_eq!(mana.current, 100.0);
        assert!(mana.is_full());
    }

    #[test]
    fn test_speed_positive_values() {
        let speed = Speed::new(-5.0);
        assert_eq!(speed.0, 0.0); // Negative values clamped to 0

        let positive_speed = Speed::new(10.0);
        assert_eq!(positive_speed.0, 10.0);
    }

    #[test]
    fn test_energy_pool_operations() {
        let mut energy = EnergyPool::new_full(100.0);

        // Test spend (consume)
        assert!(energy.spend(25.0));
        assert_eq!(energy.current, 75.0);
        assert!(!energy.is_empty());

        // Test failed spend
        assert!(!energy.spend(80.0));
        assert_eq!(energy.current, 75.0);

        // Test deplete (forced drain)
        energy.deplete(20.0);
        assert_eq!(energy.current, 55.0);

        // Test restore
        energy.restore(15.0);
        assert_eq!(energy.current, 70.0);
        assert_eq!(energy.percentage(), 0.7);
    }

    #[test]
    fn test_area_effect_types() {
        let settings = GameSettings::default();

        let magic = AreaEffectType::Magic;
        assert_eq!(magic.damage_per_second(&settings).0, 150.0);
        assert_eq!(magic.radius(&settings).0, 3.0);
        assert_eq!(magic.duration(&settings), 2.0);

        let poison = AreaEffectType::Poison;
        assert_eq!(poison.damage_per_second(&settings).0, 80.0);
        assert_eq!(poison.radius(&settings).0, 4.0);
        assert_eq!(poison.duration(&settings), 4.0);
    }
}
