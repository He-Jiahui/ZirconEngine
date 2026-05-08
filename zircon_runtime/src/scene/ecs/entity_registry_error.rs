use thiserror::Error;

use crate::scene::ecs::InternalEntity;
use crate::scene::EntityId;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum EntityRegistryError {
    #[error("stable scene entity {0} is already registered")]
    DuplicateStableId(EntityId),
    #[error("stable scene entity {0} is not registered")]
    MissingStableId(EntityId),
    #[error("internal scene entity {0:?} is stale or unknown")]
    InvalidInternalEntity(InternalEntity),
}
