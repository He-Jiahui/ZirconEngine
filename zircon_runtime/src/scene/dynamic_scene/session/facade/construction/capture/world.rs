use crate::scene::World;

use super::super::super::super::construction;
use super::super::super::super::*;

impl RuntimeSessionArchive {
    pub fn from_world(
        slot_id: impl Into<String>,
        world: &World,
    ) -> Result<Self, RuntimeSessionArchiveError> {
        construction::from_world(slot_id, world)
    }

    pub fn from_world_with_metadata(
        slot_id: impl Into<String>,
        world: &World,
        metadata: RuntimeSessionMetadata,
    ) -> Result<Self, RuntimeSessionArchiveError> {
        construction::from_world_with_metadata(slot_id, world, metadata)
    }
}
