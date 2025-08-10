use derive_more::{Display, From};
use serde::{Deserialize, Serialize};

/// A movement speed value constrained to [0.1, 50.0]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Display, From, Serialize, Deserialize)]
pub struct MovementSpeed(f32);

impl MovementSpeed {
    const MIN: f32 = 0.1;
    const MAX: f32 = 50.0;

    pub fn new(value: f32) -> Self {
        Self(value.clamp(Self::MIN, Self::MAX))
    }

    pub fn get(self) -> f32 {
        self.0
    }
}

impl Default for MovementSpeed {
    fn default() -> Self {
        Self::new(5.0)
    }
}

/// A health value constrained to [1.0, 1000.0]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Display, From, Serialize, Deserialize)]
pub struct HealthValue(f32);

impl HealthValue {
    const MIN: f32 = 1.0;
    const MAX: f32 = 1000.0;

    pub fn new(value: f32) -> Self {
        Self(value.clamp(Self::MIN, Self::MAX))
    }

    pub fn get(self) -> f32 {
        self.0
    }
}

impl Default for HealthValue {
    fn default() -> Self {
        Self::new(100.0)
    }
}

/// A mana value constrained to [1.0, 500.0]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Display, From, Serialize, Deserialize)]
pub struct ManaValue(f32);

impl ManaValue {
    const MIN: f32 = 1.0;
    const MAX: f32 = 500.0;

    pub fn new(value: f32) -> Self {
        Self(value.clamp(Self::MIN, Self::MAX))
    }

    pub fn get(self) -> f32 {
        self.0
    }
}

impl Default for ManaValue {
    fn default() -> Self {
        Self::new(50.0)
    }
}

/// An energy value constrained to [1.0, 500.0]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Display, From, Serialize, Deserialize)]
pub struct EnergyValue(f32);

impl EnergyValue {
    const MIN: f32 = 1.0;
    const MAX: f32 = 500.0;

    pub fn new(value: f32) -> Self {
        Self(value.clamp(Self::MIN, Self::MAX))
    }

    pub fn get(self) -> f32 {
        self.0
    }
}

impl Default for EnergyValue {
    fn default() -> Self {
        Self::new(100.0)
    }
}

/// A bullet speed value constrained to [1.0, 100.0]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Display, From, Serialize, Deserialize)]
pub struct BulletSpeed(f32);

impl BulletSpeed {
    const MIN: f32 = 1.0;
    const MAX: f32 = 100.0;

    pub fn new(value: f32) -> Self {
        Self(value.clamp(Self::MIN, Self::MAX))
    }

    pub fn get(self) -> f32 {
        self.0
    }
}

impl Default for BulletSpeed {
    fn default() -> Self {
        Self::new(15.0)
    }
}

/// A bullet damage value constrained to [0.1, 50.0]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Display, From, Serialize, Deserialize)]
pub struct BulletDamage(f32);

impl BulletDamage {
    const MIN: f32 = 0.1;
    const MAX: f32 = 50.0;

    pub fn new(value: f32) -> Self {
        Self(value.clamp(Self::MIN, Self::MAX))
    }

    pub fn get(self) -> f32 {
        self.0
    }
}

impl Default for BulletDamage {
    fn default() -> Self {
        Self::new(2.0)
    }
}

/// A lifetime value constrained to [0.5, 10.0]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Display, From, Serialize, Deserialize)]
pub struct Lifetime(f32);

impl Lifetime {
    const MIN: f32 = 0.5;
    const MAX: f32 = 10.0;

    pub fn new(value: f32) -> Self {
        Self(value.clamp(Self::MIN, Self::MAX))
    }

    pub fn get(self) -> f32 {
        self.0
    }
}

impl Default for Lifetime {
    fn default() -> Self {
        Self::new(3.0)
    }
}

/// A damage per second value constrained to [1.0, 1000.0]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Display, From, Serialize, Deserialize)]
pub struct DamagePerSecond(f32);

impl DamagePerSecond {
    const MIN: f32 = 1.0;
    const MAX: f32 = 1000.0;

    pub fn new(value: f32) -> Self {
        Self(value.clamp(Self::MIN, Self::MAX))
    }

    pub fn get(self) -> f32 {
        self.0
    }
}

impl Default for DamagePerSecond {
    fn default() -> Self {
        Self::new(150.0)
    }
}

/// An area radius value constrained to [0.5, 20.0]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Display, From, Serialize, Deserialize)]
pub struct AreaRadius(f32);

impl AreaRadius {
    const MIN: f32 = 0.5;
    const MAX: f32 = 20.0;

    pub fn new(value: f32) -> Self {
        Self(value.clamp(Self::MIN, Self::MAX))
    }

    pub fn get(self) -> f32 {
        self.0
    }
}

impl Default for AreaRadius {
    fn default() -> Self {
        Self::new(3.0)
    }
}

/// An area duration value constrained to [0.1, 30.0]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Display, From, Serialize, Deserialize)]
pub struct AreaDuration(f32);

impl AreaDuration {
    const MIN: f32 = 0.1;
    const MAX: f32 = 30.0;

    pub fn new(value: f32) -> Self {
        Self(value.clamp(Self::MIN, Self::MAX))
    }

    pub fn get(self) -> f32 {
        self.0
    }
}

impl Default for AreaDuration {
    fn default() -> Self {
        Self::new(2.0)
    }
}

/// A chase distance value constrained to [1.0, 50.0]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Display, From, Serialize, Deserialize)]
pub struct ChaseDistance(f32);

impl ChaseDistance {
    const MIN: f32 = 1.0;
    const MAX: f32 = 50.0;

    pub fn new(value: f32) -> Self {
        Self(value.clamp(Self::MIN, Self::MAX))
    }

    pub fn get(self) -> f32 {
        self.0
    }
}

impl Default for ChaseDistance {
    fn default() -> Self {
        Self::new(8.0)
    }
}

/// A collision distance value constrained to [0.1, 5.0]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Display, From, Serialize, Deserialize)]
pub struct CollisionDistance(f32);

impl CollisionDistance {
    const MIN: f32 = 0.1;
    const MAX: f32 = 5.0;

    pub fn new(value: f32) -> Self {
        Self(value.clamp(Self::MIN, Self::MAX))
    }

    pub fn get(self) -> f32 {
        self.0
    }
}

impl Default for CollisionDistance {
    fn default() -> Self {
        Self::new(1.2)
    }
}

/// A spawn distance minimum value constrained to [1.0, 20.0]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Display, From, Serialize, Deserialize)]
pub struct SpawnDistanceMin(f32);

impl SpawnDistanceMin {
    const MIN: f32 = 1.0;
    const MAX: f32 = 20.0;

    pub fn new(value: f32) -> Self {
        Self(value.clamp(Self::MIN, Self::MAX))
    }

    pub fn get(self) -> f32 {
        self.0
    }
}

impl Default for SpawnDistanceMin {
    fn default() -> Self {
        Self::new(5.0)
    }
}

/// A spawn distance maximum value constrained to [5.0, 50.0]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Display, From, Serialize, Deserialize)]
pub struct SpawnDistanceMax(f32);

impl SpawnDistanceMax {
    const MIN: f32 = 5.0;
    const MAX: f32 = 50.0;

    pub fn new(value: f32) -> Self {
        Self(value.clamp(Self::MIN, Self::MAX))
    }

    pub fn get(self) -> f32 {
        self.0
    }
}

impl Default for SpawnDistanceMax {
    fn default() -> Self {
        Self::new(10.5)
    }
}

/// A stopping distance value constrained to [0.1, 2.0]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Display, From, Serialize, Deserialize)]
pub struct StoppingDistance(f32);

impl StoppingDistance {
    const MIN: f32 = 0.1;
    const MAX: f32 = 2.0;

    pub fn new(value: f32) -> Self {
        Self(value.clamp(Self::MIN, Self::MAX))
    }

    pub fn get(self) -> f32 {
        self.0
    }
}

impl Default for StoppingDistance {
    fn default() -> Self {
        Self::new(0.5)
    }
}

/// A slowdown distance value constrained to [0.5, 10.0]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Display, From, Serialize, Deserialize)]
pub struct SlowdownDistance(f32);

impl SlowdownDistance {
    const MIN: f32 = 0.5;
    const MAX: f32 = 10.0;

    pub fn new(value: f32) -> Self {
        Self(value.clamp(Self::MIN, Self::MAX))
    }

    pub fn get(self) -> f32 {
        self.0
    }
}

impl Default for SlowdownDistance {
    fn default() -> Self {
        Self::new(2.0)
    }
}

/// A speed multiplier value constrained to [0.1, 1.0]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Display, From, Serialize, Deserialize)]
pub struct SpeedMultiplier(f32);

impl SpeedMultiplier {
    const MIN: f32 = 0.1;
    const MAX: f32 = 1.0;

    pub fn new(value: f32) -> Self {
        Self(value.clamp(Self::MIN, Self::MAX))
    }

    pub fn get(self) -> f32 {
        self.0
    }
}

impl Default for SpeedMultiplier {
    fn default() -> Self {
        Self::new(0.8)
    }
}

/// An LOD distance value constrained to [1.0, 50.0]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Display, From, Serialize, Deserialize)]
pub struct LodDistance(f32);

impl LodDistance {
    const MIN: f32 = 1.0;
    const MAX: f32 = 50.0;

    pub fn new(value: f32) -> Self {
        Self(value.clamp(Self::MIN, Self::MAX))
    }

    pub fn get(self) -> f32 {
        self.0
    }
}

impl Default for LodDistance {
    fn default() -> Self {
        Self::new(5.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_movement_speed_clamping() {
        assert_eq!(MovementSpeed::new(-1.0).get(), 0.1);
        assert_eq!(MovementSpeed::new(0.05).get(), 0.1);
        assert_eq!(MovementSpeed::new(5.0).get(), 5.0);
        assert_eq!(MovementSpeed::new(100.0).get(), 50.0);
    }

    #[test]
    fn test_health_value_clamping() {
        assert_eq!(HealthValue::new(0.5).get(), 1.0);
        assert_eq!(HealthValue::new(100.0).get(), 100.0);
        assert_eq!(HealthValue::new(2000.0).get(), 1000.0);
    }

    #[test]
    fn test_display() {
        let speed = MovementSpeed::new(5.5);
        assert_eq!(format!("{speed}"), "5.5");
    }

    #[test]
    fn test_defaults() {
        assert_eq!(MovementSpeed::default().get(), 5.0);
        assert_eq!(HealthValue::default().get(), 100.0);
        assert_eq!(BulletSpeed::default().get(), 15.0);
    }
}
