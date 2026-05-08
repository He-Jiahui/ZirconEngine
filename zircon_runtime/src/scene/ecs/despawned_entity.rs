use crate::scene::ecs::{EntityLocation, InternalEntity};
use crate::scene::EntityId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DespawnedEntity {
    pub stable_id: EntityId,
    pub internal: InternalEntity,
    pub location: EntityLocation,
}
