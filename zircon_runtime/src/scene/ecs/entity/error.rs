use thiserror::Error;

use crate::scene::EntityId;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum EntityRegistryError {
    #[error("stable scene entity {0} is already registered")]
    DuplicateStableId(EntityId),
    #[error("stable scene entity {0} is not registered")]
    MissingStableId(EntityId),
    #[error("scene entity registry exhausted its {max_slots} valid slots")]
    SlotCapacityExhausted { max_slots: u32 },
    #[error("an internal scene entity handle is stale or unknown")]
    InvalidInternalEntity,
}
