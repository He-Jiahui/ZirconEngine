use std::path::Path;

use super::super::super::super::super::{
    io, RuntimeSessionArchiveError, RuntimeSessionSlotImportPreviewReport,
    RuntimeSessionSlotSelector,
};

pub(in crate::scene::dynamic_scene::session) fn preview_copy_selected_slot_from_path(
    path: impl AsRef<Path>,
    selector: RuntimeSessionSlotSelector,
    new_slot_id: impl Into<String>,
) -> Result<RuntimeSessionSlotImportPreviewReport, RuntimeSessionArchiveError> {
    io::load_from_path(path)?.preview_copy_selected_slot(selector, new_slot_id)
}
