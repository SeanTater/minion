use crate::game_logic::errors::MinionError;
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

/// Trait for entities that have resource pools (health, mana, energy)
pub trait HasResources {
    fn health(&self) -> &HealthPool;
    fn mana(&self) -> &ManaPool;
    fn energy(&self) -> &EnergyPool;

    fn health_mut(&mut self) -> &mut HealthPool;
    fn mana_mut(&mut self) -> &mut ManaPool;
    fn energy_mut(&mut self) -> &mut EnergyPool;

    /// Get resource values by type
    fn get_resource(&self, resource_type: ResourceType) -> (f32, f32) {
        match resource_type {
            ResourceType::Health => (self.health().current, self.health().max),
            ResourceType::Mana => (self.mana().current, self.mana().max),
            ResourceType::Energy => (self.energy().current, self.energy().max),
        }
    }
}

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

impl HasResources for Player {
    fn health(&self) -> &HealthPool {
        &self.health
    }
    fn mana(&self) -> &ManaPool {
        &self.mana
    }
    fn energy(&self) -> &EnergyPool {
        &self.energy
    }

    fn health_mut(&mut self) -> &mut HealthPool {
        &mut self.health
    }
    fn mana_mut(&mut self) -> &mut ManaPool {
        &mut self.mana
    }
    fn energy_mut(&mut self) -> &mut EnergyPool {
        &mut self.energy
    }
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

impl HasResources for Enemy {
    fn health(&self) -> &HealthPool {
        &self.health
    }
    fn mana(&self) -> &ManaPool {
        &self.mana
    }
    fn energy(&self) -> &EnergyPool {
        &self.energy
    }

    fn health_mut(&mut self) -> &mut HealthPool {
        &mut self.health
    }
    fn mana_mut(&mut self) -> &mut ManaPool {
        &mut self.mana
    }
    fn energy_mut(&mut self) -> &mut EnergyPool {
        &mut self.energy
    }
}

#[derive(Component)]
pub struct Name(pub String);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LodLevel {
    High,   // Original high-poly model
    Medium, // Medium-poly model
    Low,    // Low-poly model
}

impl TryFrom<&str> for LodLevel {
    type Error = MinionError;

    fn try_from(config_str: &str) -> Result<Self, Self::Error> {
        match config_str {
            "high" => Ok(LodLevel::High),
            "medium" => Ok(LodLevel::Medium),
            "low" => Ok(LodLevel::Low),
            _ => Err(MinionError::InvalidConfig {
                reason: format!(
                    "Invalid LOD level '{}', expected 'high', 'medium', or 'low'",
                    config_str
                ),
            }),
        }
    }
}

impl LodLevel {
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
            AreaEffectType::Magic => Damage::new(settings.magic_damage_per_second.get()),
            AreaEffectType::Poison => Damage::new(settings.poison_damage_per_second.get()),
        }
    }

    pub fn radius(&self, settings: &GameSettings) -> Distance {
        match self {
            AreaEffectType::Magic => Distance::new(settings.magic_area_radius.get()),
            AreaEffectType::Poison => Distance::new(settings.poison_area_radius.get()),
        }
    }

    pub fn duration(&self, settings: &GameSettings) -> f32 {
        match self {
            AreaEffectType::Magic => settings.magic_area_duration.get(),
            AreaEffectType::Poison => settings.poison_area_duration.get(),
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

/// Type-safe wrapper for navigation paths that prevents index-out-of-bounds errors
#[derive(Debug, Clone, PartialEq)]
pub struct NavPath {
    waypoints: Vec<Vec3>,
    current_index: usize,
}

impl NavPath {
    /// Create a new empty navigation path
    pub fn new() -> Self {
        Self {
            waypoints: Vec::new(),
            current_index: 0,
        }
    }

    /// Create a navigation path from a vector of waypoints
    pub fn from_waypoints(waypoints: Vec<Vec3>) -> Self {
        Self {
            waypoints,
            current_index: 0,
        }
    }

    /// Get the current waypoint the agent should move towards
    pub fn current_waypoint(&self) -> Option<Vec3> {
        self.waypoints.get(self.current_index).copied()
    }

    /// Advance to the next waypoint in the path
    pub fn advance(&mut self) -> bool {
        if self.current_index < self.waypoints.len() {
            self.current_index += 1;
            true
        } else {
            false
        }
    }

    /// Check if the path is complete (no more waypoints)
    pub fn is_complete(&self) -> bool {
        self.current_index >= self.waypoints.len()
    }

    /// Check if the path has any waypoints
    pub fn is_empty(&self) -> bool {
        self.waypoints.is_empty()
    }

    /// Check if there's a valid current waypoint
    pub fn has_current_waypoint(&self) -> bool {
        !self.is_empty() && !self.is_complete()
    }

    /// Get the total number of waypoints in the path
    pub fn len(&self) -> usize {
        self.waypoints.len()
    }

    /// Get the current waypoint index
    pub fn current_index(&self) -> usize {
        self.current_index
    }

    /// Get the remaining waypoints count
    pub fn remaining_waypoints(&self) -> usize {
        if self.current_index < self.waypoints.len() {
            self.waypoints.len() - self.current_index
        } else {
            0
        }
    }

    /// Clear all waypoints and reset to beginning
    pub fn clear(&mut self) {
        self.waypoints.clear();
        self.current_index = 0;
    }

    /// Get the final destination (last waypoint)
    pub fn final_destination(&self) -> Option<Vec3> {
        self.waypoints.last().copied()
    }

    /// Reset to the beginning of the path
    pub fn reset(&mut self) {
        self.current_index = 0;
    }

    /// Get all waypoints (for debugging or advanced usage)
    pub fn waypoints(&self) -> &[Vec3] {
        &self.waypoints
    }
}

impl Default for NavPath {
    fn default() -> Self {
        Self::new()
    }
}

/// Pathfinding agent component that can be used by both players and enemies
#[derive(Component)]
pub struct PathfindingAgent {
    /// Current navigation path with type-safe waypoint management
    pub nav_path: NavPath,
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
            nav_path: NavPath::new(),
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
        self.nav_path.current_waypoint()
    }

    /// Check if the agent has a valid path to follow
    pub fn has_path(&self) -> bool {
        self.nav_path.has_current_waypoint()
    }

    /// Clear the current path
    pub fn clear_path(&mut self) {
        self.nav_path.clear();
    }

    /// Set a new path for the agent
    pub fn set_path(&mut self, path: Vec<Vec3>) {
        self.nav_path = NavPath::from_waypoints(path);
    }

    /// Advance to the next waypoint in the path
    pub fn advance_waypoint(&mut self) {
        self.nav_path.advance();
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

        // Test magic area effect
        let magic = AreaEffectType::Magic;
        assert!(magic.damage_per_second(&settings).0 > 0.0);
        assert!(magic.radius(&settings).0 > 0.0);
        assert!(magic.duration(&settings) > 0.0);

        // Test poison area effect
        let poison = AreaEffectType::Poison;
        assert!(poison.damage_per_second(&settings).0 > 0.0);
        assert!(poison.radius(&settings).0 > 0.0);
        assert!(poison.duration(&settings) > 0.0);
    }

    #[test]
    fn test_has_resources_trait() {
        // Test Player implements HasResources
        let mut player = Player {
            move_target: None,
            speed: Speed::new(5.0),
            health: HealthPool::new_full(100.0),
            mana: ManaPool::new_full(50.0),
            energy: EnergyPool::new_full(75.0),
        };

        // Test resource access
        assert_eq!(player.get_resource(ResourceType::Health), (100.0, 100.0));
        assert_eq!(player.get_resource(ResourceType::Mana), (50.0, 50.0));
        assert_eq!(player.get_resource(ResourceType::Energy), (75.0, 75.0));

        // Test mutable access
        player.health_mut().take_damage(Damage::new(20.0));
        assert_eq!(player.get_resource(ResourceType::Health), (80.0, 100.0));

        // Test Enemy implements HasResources
        let mut enemy = Enemy {
            speed: Speed::new(3.0),
            health: HealthPool::new_full(80.0),
            mana: ManaPool::new_full(30.0),
            energy: EnergyPool::new_full(40.0),
            chase_distance: Distance::new(10.0),
            is_dying: false,
        };

        // Test resource access
        assert_eq!(enemy.get_resource(ResourceType::Health), (80.0, 80.0));
        assert_eq!(enemy.get_resource(ResourceType::Mana), (30.0, 30.0));
        assert_eq!(enemy.get_resource(ResourceType::Energy), (40.0, 40.0));

        // Test mutable access
        enemy.mana_mut().spend(10.0);
        assert_eq!(enemy.get_resource(ResourceType::Mana), (20.0, 30.0));
    }

    #[test]
    fn test_nav_path_empty() {
        let path = NavPath::new();
        assert!(path.is_empty());
        assert!(path.is_complete());
        assert!(!path.has_current_waypoint());
        assert_eq!(path.current_waypoint(), None);
        assert_eq!(path.len(), 0);
        assert_eq!(path.current_index(), 0);
        assert_eq!(path.remaining_waypoints(), 0);
        assert_eq!(path.final_destination(), None);
    }

    #[test]
    fn test_nav_path_with_waypoints() {
        let waypoints = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(2.0, 0.0, 0.0),
        ];
        let mut path = NavPath::from_waypoints(waypoints.clone());

        assert!(!path.is_empty());
        assert!(!path.is_complete());
        assert!(path.has_current_waypoint());
        assert_eq!(path.len(), 3);
        assert_eq!(path.current_index(), 0);
        assert_eq!(path.remaining_waypoints(), 3);
        assert_eq!(path.current_waypoint(), Some(Vec3::new(0.0, 0.0, 0.0)));
        assert_eq!(path.final_destination(), Some(Vec3::new(2.0, 0.0, 0.0)));

        // Advance through waypoints
        assert!(path.advance());
        assert_eq!(path.current_index(), 1);
        assert_eq!(path.remaining_waypoints(), 2);
        assert_eq!(path.current_waypoint(), Some(Vec3::new(1.0, 0.0, 0.0)));

        assert!(path.advance());
        assert_eq!(path.current_index(), 2);
        assert_eq!(path.remaining_waypoints(), 1);
        assert_eq!(path.current_waypoint(), Some(Vec3::new(2.0, 0.0, 0.0)));

        assert!(path.advance());
        assert_eq!(path.current_index(), 3);
        assert_eq!(path.remaining_waypoints(), 0);
        assert_eq!(path.current_waypoint(), None);
        assert!(path.is_complete());
        assert!(!path.has_current_waypoint());

        // Try to advance past end
        assert!(!path.advance());
        assert_eq!(path.current_index(), 3);
    }

    #[test]
    fn test_nav_path_clear_and_reset() {
        let waypoints = vec![Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0)];
        let mut path = NavPath::from_waypoints(waypoints);

        // Advance to second waypoint
        path.advance();
        assert_eq!(path.current_index(), 1);

        // Reset to beginning
        path.reset();
        assert_eq!(path.current_index(), 0);
        assert_eq!(path.current_waypoint(), Some(Vec3::new(0.0, 0.0, 0.0)));

        // Clear the path
        path.clear();
        assert!(path.is_empty());
        assert_eq!(path.current_index(), 0);
        assert_eq!(path.current_waypoint(), None);
    }

    #[test]
    fn test_pathfinding_agent_with_nav_path() {
        let mut agent = PathfindingAgent::new();
        assert!(!agent.has_path());
        assert_eq!(agent.current_waypoint(), None);

        let waypoints = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(2.0, 0.0, 0.0),
        ];
        agent.set_path(waypoints);

        assert!(agent.has_path());
        assert_eq!(agent.current_waypoint(), Some(Vec3::new(0.0, 0.0, 0.0)));

        agent.advance_waypoint();
        assert_eq!(agent.current_waypoint(), Some(Vec3::new(1.0, 0.0, 0.0)));

        agent.advance_waypoint();
        assert_eq!(agent.current_waypoint(), Some(Vec3::new(2.0, 0.0, 0.0)));

        agent.advance_waypoint();
        assert_eq!(agent.current_waypoint(), None);
        assert!(!agent.has_path());

        agent.clear_path();
        assert!(!agent.has_path());
        assert_eq!(agent.current_waypoint(), None);
    }
}
