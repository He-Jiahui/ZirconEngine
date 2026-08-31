use std::fmt;

use crate::scene::EntityId;

use super::internal::InternalEntity;
use super::location::EntityLocation;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct StableEntityLocation {
    pub(crate) stable_id: EntityId,
    pub(crate) internal: InternalEntity,
    pub(crate) location: EntityLocation,
}

impl fmt::Debug for StableEntityLocation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("StableEntityLocation")
            .field("stable_id", &self.stable_id)
            .field("location", &self.location)
            .finish()
    }
}

impl StableEntityLocation {
    pub const fn stable_id(self) -> EntityId {
        self.stable_id
    }
}
