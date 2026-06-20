use crate::scene::EntityId;

use super::internal::InternalEntity;
use super::location::EntityLocation;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DespawnedEntity {
    pub stable_id: EntityId,
    pub internal: InternalEntity,
    pub location: EntityLocation,
}
