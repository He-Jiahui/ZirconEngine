use std::path::Path;

use crate::scene::World;

use super::super::super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError,
    RuntimeSessionArchiveRetentionPolicy, RuntimeSessionMetadata, RuntimeSessionSlotSelector, io,
};

impl RuntimeSessionArchive {
    pub fn preview_capture_world_selected_slot_with_tag_retention_to_path(
        path: impl AsRef<Path>,
        tag: &str,
        selector: RuntimeSessionSlotSelector,
        world: &World,
        metadata: RuntimeSessionMetadata,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError> {
        io::load_or_empty_from_path(path)?.preview_capture_world_selected_slot_with_tag_retention(
            tag, selector, world, metadata, policy,
        )
    }
}
