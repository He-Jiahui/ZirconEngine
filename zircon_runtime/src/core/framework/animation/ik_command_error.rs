use thiserror::Error;

use crate::core::framework::scene::{EntityId, WorldHandle};

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum AnimationIkCommandError {
    #[error("animation manager does not support IK commands")]
    Unsupported,
    #[error("IK command for entity {entity} in {world:?} contains non-finite input")]
    NonFiniteInput {
        world: WorldHandle,
        entity: EntityId,
    },
    #[error("IK command for entity {entity} in {world:?} has weight outside [0, 1]")]
    InvalidWeight {
        world: WorldHandle,
        entity: EntityId,
    },
    #[error("look-at command for entity {entity} in {world:?} has a degenerate local axis")]
    DegenerateAxis {
        world: WorldHandle,
        entity: EntityId,
    },
    #[error("IK command queue for {world:?} reached its {capacity}-command capacity")]
    QueueFull { world: WorldHandle, capacity: usize },
}
