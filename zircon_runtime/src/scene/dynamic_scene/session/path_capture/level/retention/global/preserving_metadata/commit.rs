use std::path::Path;

use crate::scene::LevelSystem;

use super::super::super::super::super::super::{
    io, RuntimeSessionArchive, RuntimeSessionArchiveCaptureRetentionReport,
    RuntimeSessionArchiveError, RuntimeSessionArchiveRetentionPolicy, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    pub fn capture_level_selected_slot_preserving_metadata_with_retention_to_path_atomically(
        path: impl AsRef<Path>,
        selector: RuntimeSessionSlotSelector,
        level: &LevelSystem,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError> {
        let path = path.as_ref();
        let mut archive = io::load_or_empty_from_path(path)?;
        let report = archive.capture_level_selected_slot_preserving_metadata_with_retention(
            selector, level, policy,
        )?;
        io::save_to_path_atomically(&archive, path)?;
        Ok(report)
    }
}
