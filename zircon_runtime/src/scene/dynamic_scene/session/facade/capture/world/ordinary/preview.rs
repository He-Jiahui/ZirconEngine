use crate::scene::World;

use super::super::super::super::super::*;

impl RuntimeSessionArchive {
    pub fn preview_capture_world_slot(
        &self,
        slot_id: impl Into<String>,
        world: &World,
        metadata: RuntimeSessionMetadata,
    ) -> Result<RuntimeSessionSlotCapturePreviewReport, RuntimeSessionArchiveError> {
        slot_capture::preview_world_slot(self, slot_id, world, metadata)
            .map(|preview| preview.report)
    }
}
