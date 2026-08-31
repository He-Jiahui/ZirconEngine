use std::path::Path;

use crate::scene::LevelSystem;

use super::super::super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError,
    RuntimeSessionArchiveRetentionPolicy, io,
};

impl RuntimeSessionArchive {
    pub fn preview_capture_level_slot_with_tag_retention_to_path(
        path: impl AsRef<Path>,
        tag: &str,
        slot_id: impl Into<String>,
        level: &LevelSystem,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError> {
        io::load_or_empty_from_path(path)?
            .preview_capture_level_slot_with_tag_retention(tag, slot_id, level, policy)
    }
}
