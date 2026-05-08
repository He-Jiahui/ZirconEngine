use thiserror::Error;

use crate::scene::ecs::{ComponentId, StorageType};

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum StorageError {
    #[error("component {component_id:?} is already registered as {existing:?}, not {requested:?}")]
    StorageTypeMismatch {
        component_id: ComponentId,
        existing: StorageType,
        requested: StorageType,
    },
    #[error("stored component {component_id:?} has a different Rust type")]
    ComponentTypeMismatch { component_id: ComponentId },
}
