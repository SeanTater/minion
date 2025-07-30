use bevy::prelude::*;
use derive_more::{Add, Mul, Display, From};
use std::ops::Sub;

// Compound resource pools that bundle current and max values
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct HealthPool {
    pub current: f32,
    pub max: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct ManaPool {
    pub current: f32,
    pub max: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Mul, Display, From)]
pub struct Speed(pub f32);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Add, Mul, Display, From)]
pub struct Distance(pub f32);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Add, Mul, Display, From)]
pub struct Damage(pub f32);

impl HealthPool {
    pub fn new(current: f32, max: f32) -> Self {
        Self {
            current: current.max(0.0).min(max),
            max: max.max(0.0),
        }
    }
    
    pub fn new_full(max: f32) -> Self {
        Self::new(max, max)
    }
    
    pub fn is_dead(self) -> bool { self.current <= 0.0 }
    pub fn is_full(self) -> bool { self.current >= self.max }
    
    pub fn take_damage(&mut self, damage: Damage) {
        self.current = (self.current - damage.0).max(0.0);
    }
    
    pub fn heal(&mut self, amount: f32) {
        self.current = (self.current + amount).min(self.max);
    }
    
    pub fn percentage(self) -> f32 {
        if self.max > 0.0 { self.current / self.max } else { 0.0 }
    }
}

impl ManaPool {
    pub fn new(current: f32, max: f32) -> Self {
        Self {
            current: current.max(0.0).min(max),
            max: max.max(0.0),
        }
    }
    
    pub fn new_full(max: f32) -> Self {
        Self::new(max, max)
    }
    
    pub fn is_empty(self) -> bool { self.current <= 0.0 }
    pub fn is_full(self) -> bool { self.current >= self.max }
    
    pub fn spend(&mut self, cost: f32) -> bool {
        if self.current >= cost {
            self.current -= cost;
            true
        } else {
            false
        }
    }
    
    pub fn restore(&mut self, amount: f32) {
        self.current = (self.current + amount).min(self.max);
    }
    
    pub fn percentage(self) -> f32 {
        if self.max > 0.0 { self.current / self.max } else { 0.0 }
    }
}

impl std::fmt::Display for HealthPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.0}/{:.0}", self.current, self.max)
    }
}

impl std::fmt::Display for ManaPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.0}/{:.0}", self.current, self.max)
    }
}

impl Speed {
    pub fn new(value: f32) -> Self { Self(value.max(0.0)) }
    pub const ZERO: Speed = Speed(0.0);
}

impl Distance {
    pub fn new(value: f32) -> Self { Self(value.max(0.0)) }
    pub const ZERO: Distance = Distance(0.0);
}

// Manual implementations for operations not available in derive_more

impl Sub for Distance {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output { Self((self.0 - rhs.0).max(0.0)) }
}

impl Sub for Damage {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output { Self((self.0 - rhs.0).max(0.0)) }
}

impl std::ops::Div<f32> for Distance {
    type Output = Self;
    fn div(self, rhs: f32) -> Self::Output { Self(self.0 / rhs) }
}

impl Damage {
    pub fn new(value: f32) -> Self { Self(value.max(0.0)) }
    pub const ZERO: Damage = Damage(0.0);
}

// Custom math operations for Vec3 * Speed
impl std::ops::Mul<Speed> for Vec3 {
    type Output = Vec3;
    fn mul(self, rhs: Speed) -> Self::Output { self * rhs.0 }
}

// Custom math operations for f32 comparisons
impl PartialOrd<f32> for Distance {
    fn partial_cmp(&self, other: &f32) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl PartialEq<f32> for Distance {
    fn eq(&self, other: &f32) -> bool { self.0 == *other }
}


#[derive(Component)]
pub struct Player {
    pub move_target: Option<Vec3>,
    pub speed: Speed,
    pub health: HealthPool,
    pub mana: ManaPool,
}

#[derive(Component)]
pub struct Ground;

#[derive(Component)]
pub struct CameraFollow {
    pub offset: Vec3,
}

#[derive(Component)]
pub struct Enemy {
    pub speed: Speed,
    pub health: HealthPool,
    pub chase_distance: Distance,
    pub is_dying: bool,
}

#[derive(Component)]
pub struct Name(pub String);

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
    pub fn damage_per_second(&self) -> Damage {
        match self {
            AreaEffectType::Magic => Damage::new(150.0),
            AreaEffectType::Poison => Damage::new(80.0),
        }
    }
    
    pub fn radius(&self) -> Distance {
        match self {
            AreaEffectType::Magic => Distance::new(3.0),
            AreaEffectType::Poison => Distance::new(4.0),
        }
    }
    
    pub fn duration(&self) -> f32 {
        match self {
            AreaEffectType::Magic => 2.0,
            AreaEffectType::Poison => 4.0,
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
    fn test_area_effect_types() {
        let magic = AreaEffectType::Magic;
        assert_eq!(magic.damage_per_second().0, 150.0);
        assert_eq!(magic.radius().0, 3.0);
        
        let poison = AreaEffectType::Poison;
        assert_eq!(poison.damage_per_second().0, 80.0);
        assert_eq!(poison.radius().0, 4.0);
    }
}