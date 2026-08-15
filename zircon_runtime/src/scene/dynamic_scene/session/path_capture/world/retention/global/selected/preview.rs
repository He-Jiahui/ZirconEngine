use std::path::Path;

use crate::scene::World;

use super::super::super::super::super::super::{
    io, RuntimeSessionArchive, RuntimeSessionArchiveCaptureRetentionReport,
    RuntimeSessionArchiveError, RuntimeSessionArchiveRetentionPolicy, RuntimeSessionMetadata,
    RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    pub fn preview_capture_world_selected_slot_with_retention_to_path(
        path: impl AsRef<Path>,
        selector: RuntimeSessionSlotSelector,
        world: &World,
        metadata: RuntimeSessionMetadata,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError> {
        io::load_or_empty_from_path(path)?
            .preview_capture_world_selected_slot_with_retention(selector, world, metadata, policy)
    }
}
