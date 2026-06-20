use std::path::Path;

use crate::scene::LevelSystem;

use super::super::super::super::{
    io, RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotCapturePreviewReport,
};

impl RuntimeSessionArchive {
    pub fn preview_capture_level_slot_to_path(
        path: impl AsRef<Path>,
        slot_id: impl Into<String>,
        level: &LevelSystem,
    ) -> Result<RuntimeSessionSlotCapturePreviewReport, RuntimeSessionArchiveError> {
        io::load_or_empty_from_path(path)?.preview_capture_level_slot(slot_id, level)
    }
}
