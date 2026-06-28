use crate::core::framework::navigation::NavigationError;
use crate::core::framework::script::ScriptHostError;
use crate::scene::{EntityId, SceneError};

pub(super) type GameplayHostResult<T> = std::result::Result<T, GameplayHostError>;

#[derive(Debug, thiserror::Error)]
pub(super) enum GameplayHostError {
    #[error(transparent)]
    Scene(#[from] SceneError),
    #[error(transparent)]
    Navigation(#[from] NavigationError),
    #[error("invalid JSON payload: {0}")]
    Json(#[from] serde_json::Error),
    #[error("{operation} entity {entity} is missing")]
    MissingEntity {
        operation: &'static str,
        entity: EntityId,
    },
}

impl GameplayHostError {
    pub(super) fn missing_entity(operation: &'static str, entity: EntityId) -> Self {
        Self::MissingEntity { operation, entity }
    }
}

impl From<GameplayHostError> for ScriptHostError {
    fn from(error: GameplayHostError) -> Self {
        Self::new(error.to_string())
    }
}
