use std::path::Path;

use crate::scene::World;

use super::super::super::super::{
    io, RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionMetadata,
    RuntimeSessionSlotCapturePreviewReport, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    pub fn preview_capture_world_selected_slot_to_path(
        path: impl AsRef<Path>,
        selector: RuntimeSessionSlotSelector,
        world: &World,
        metadata: RuntimeSessionMetadata,
    ) -> Result<RuntimeSessionSlotCapturePreviewReport, RuntimeSessionArchiveError> {
        io::load_or_empty_from_path(path)?
            .preview_capture_world_selected_slot(selector, world, metadata)
    }

    pub fn preview_capture_world_selected_slot_preserving_metadata_to_path(
        path: impl AsRef<Path>,
        selector: RuntimeSessionSlotSelector,
        world: &World,
    ) -> Result<RuntimeSessionSlotCapturePreviewReport, RuntimeSessionArchiveError> {
        io::load_or_empty_from_path(path)?
            .preview_capture_world_selected_slot_preserving_metadata(selector, world)
    }
}
