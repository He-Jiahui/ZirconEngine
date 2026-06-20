use std::path::Path;

use crate::scene::World;

use super::super::super::super::super::super::{
    io, RuntimeSessionArchive, RuntimeSessionArchiveCaptureRetentionReport,
    RuntimeSessionArchiveError, RuntimeSessionArchiveRetentionPolicy, RuntimeSessionMetadata,
};

impl RuntimeSessionArchive {
    pub fn preview_capture_world_slot_with_tag_retention_to_path(
        path: impl AsRef<Path>,
        tag: &str,
        slot_id: impl Into<String>,
        world: &World,
        metadata: RuntimeSessionMetadata,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError> {
        io::load_or_empty_from_path(path)?
            .preview_capture_world_slot_with_tag_retention(tag, slot_id, world, metadata, policy)
    }
}
