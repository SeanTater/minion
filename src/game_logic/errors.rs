use thiserror::Error;
use bevy::prelude::*;

#[derive(Error, Debug)]
pub enum GameError {
    #[error("Entity not found: {entity:?}")]
    EntityNotFound { entity: Entity },
    
    #[error("Component missing from entity {entity:?}: {component}")]
    ComponentMissing { entity: Entity, component: &'static str },
    
    #[error("Invalid spawn position: {position:?}")]
    InvalidSpawnPosition { position: Vec3 },
    
    #[error("Resource not available: {resource}")]
    ResourceUnavailable { resource: &'static str },
    
    #[error("Combat system error: {message}")]
    CombatError { message: String },
    
    #[error("Physics calculation failed: {operation}")]
    PhysicsError { operation: String },
}

/// Result type alias for game operations
pub type GameResult<T> = Result<T, GameError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_error_display() {
        let entity = Entity::from_raw(42);
        let err = GameError::EntityNotFound { entity };
        assert!(err.to_string().contains("Entity not found"));
        
        let err = GameError::ComponentMissing { 
            entity, 
            component: "Transform" 
        };
        assert!(err.to_string().contains("Component missing"));
        assert!(err.to_string().contains("Transform"));
        
        let err = GameError::InvalidSpawnPosition { 
            position: Vec3::new(100.0, 0.0, 100.0) 
        };
        assert!(err.to_string().contains("Invalid spawn position"));
    }
}