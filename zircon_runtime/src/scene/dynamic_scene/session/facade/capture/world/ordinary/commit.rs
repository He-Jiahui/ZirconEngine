use crate::scene::World;

use super::super::super::super::super::*;

impl RuntimeSessionArchive {
    pub fn capture_world_slot(
        &mut self,
        slot_id: impl Into<String>,
        world: &World,
        metadata: RuntimeSessionMetadata,
    ) -> Result<(), RuntimeSessionArchiveError> {
        slot_capture::capture_world_slot(self, slot_id, world, metadata)
    }
}
