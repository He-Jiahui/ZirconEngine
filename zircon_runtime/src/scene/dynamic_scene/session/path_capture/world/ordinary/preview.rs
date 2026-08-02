use std::path::Path;

use crate::scene::World;

use super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionMetadata,
    RuntimeSessionSlotCapturePreviewReport, io,
};

impl RuntimeSessionArchive {
    pub fn preview_capture_world_slot_to_path(
        path: impl AsRef<Path>,
        slot_id: impl Into<String>,
        world: &World,
        metadata: RuntimeSessionMetadata,
    ) -> Result<RuntimeSessionSlotCapturePreviewReport, RuntimeSessionArchiveError> {
        io::load_or_empty_from_path(path)?.preview_capture_world_slot(slot_id, world, metadata)
    }
}
