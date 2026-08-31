use std::path::Path;

use crate::scene::LevelSystem;

use super::super::super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError,
    RuntimeSessionArchiveRetentionPolicy, io,
};

impl RuntimeSessionArchive {
    pub fn capture_level_slot_with_tag_retention_to_path_atomically(
        path: impl AsRef<Path>,
        tag: &str,
        slot_id: impl Into<String>,
        level: &LevelSystem,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError> {
        let path = path.as_ref();
        let mut archive = io::load_or_empty_from_path(path)?;
        let report = archive.capture_level_slot_with_tag_retention(tag, slot_id, level, policy)?;
        io::save_to_path_atomically(&archive, path)?;
        Ok(report)
    }
}
