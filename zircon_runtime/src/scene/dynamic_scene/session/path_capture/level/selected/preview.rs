use std::path::Path;

use crate::scene::LevelSystem;

use super::super::super::super::{
    io, RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotCapturePreviewReport,
    RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    pub fn preview_capture_level_selected_slot_to_path(
        path: impl AsRef<Path>,
        selector: RuntimeSessionSlotSelector,
        level: &LevelSystem,
    ) -> Result<RuntimeSessionSlotCapturePreviewReport, RuntimeSessionArchiveError> {
        io::load_or_empty_from_path(path)?.preview_capture_level_selected_slot(selector, level)
    }

    pub fn preview_capture_level_selected_slot_preserving_metadata_to_path(
        path: impl AsRef<Path>,
        selector: RuntimeSessionSlotSelector,
        level: &LevelSystem,
    ) -> Result<RuntimeSessionSlotCapturePreviewReport, RuntimeSessionArchiveError> {
        io::load_or_empty_from_path(path)?
            .preview_capture_level_selected_slot_preserving_metadata(selector, level)
    }
}
