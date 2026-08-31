use crate::scene::EntityId;

use super::internal::InternalEntity;
use super::location::EntityLocation;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct DespawnedEntity {
    pub(crate) stable_id: EntityId,
    pub(crate) internal: InternalEntity,
    pub(crate) location: EntityLocation,
}
