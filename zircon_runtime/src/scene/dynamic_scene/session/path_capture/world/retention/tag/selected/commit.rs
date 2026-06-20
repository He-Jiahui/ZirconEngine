use std::path::Path;

use crate::scene::World;

use super::super::super::super::super::super::{
    io, RuntimeSessionArchive, RuntimeSessionArchiveCaptureRetentionReport,
    RuntimeSessionArchiveError, RuntimeSessionArchiveRetentionPolicy, RuntimeSessionMetadata,
    RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    pub fn capture_world_selected_slot_with_tag_retention_to_path_atomically(
        path: impl AsRef<Path>,
        tag: &str,
        selector: RuntimeSessionSlotSelector,
        world: &World,
        metadata: RuntimeSessionMetadata,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError> {
        let path = path.as_ref();
        let mut archive = io::load_or_empty_from_path(path)?;
        let report = archive.capture_world_selected_slot_with_tag_retention(
            tag, selector, world, metadata, policy,
        )?;
        io::save_to_path_atomically(&archive, path)?;
        Ok(report)
    }
}
