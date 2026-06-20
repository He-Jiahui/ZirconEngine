use crate::scene::World;

use super::super::super::super::super::super::*;

impl RuntimeSessionArchive {
    pub fn preview_capture_world_slot_with_retention(
        &self,
        slot_id: impl Into<String>,
        world: &World,
        metadata: RuntimeSessionMetadata,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError> {
        capture_retention::preview_world_slot_with_retention(self, slot_id, world, metadata, policy)
    }
}
