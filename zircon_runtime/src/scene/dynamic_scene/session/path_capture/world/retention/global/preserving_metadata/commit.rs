use std::path::Path;

use crate::scene::World;

use super::super::super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError,
    RuntimeSessionArchiveRetentionPolicy, RuntimeSessionSlotSelector, io,
};

impl RuntimeSessionArchive {
    pub fn capture_world_selected_slot_preserving_metadata_with_retention_to_path_atomically(
        path: impl AsRef<Path>,
        selector: RuntimeSessionSlotSelector,
        world: &World,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError> {
        let path = path.as_ref();
        let mut archive = io::load_or_empty_from_path(path)?;
        let report = archive.capture_world_selected_slot_preserving_metadata_with_retention(
            selector, world, policy,
        )?;
        io::save_to_path_atomically(&archive, path)?;
        Ok(report)
    }
}
