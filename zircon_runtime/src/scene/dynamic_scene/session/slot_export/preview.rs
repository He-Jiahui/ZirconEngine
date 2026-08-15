use std::path::Path;

use super::super::{
    target_path, RuntimeSessionArchive, RuntimeSessionArchiveError,
    RuntimeSessionSlotExportPreviewReport, RuntimeSessionSlotSelector,
};

pub(in crate::scene::dynamic_scene::session) fn preview_single_slot_archive(
    archive: &RuntimeSessionArchive,
    slot_id: &str,
) -> Result<RuntimeSessionSlotExportPreviewReport, RuntimeSessionArchiveError> {
    archive.ensure_supported()?;

    let slot = archive
        .slot(slot_id)
        .ok_or_else(|| RuntimeSessionArchiveError::MissingSlot {
            slot_id: slot_id.to_string(),
        })?;

    Ok(RuntimeSessionSlotExportPreviewReport {
        source_slot_id: slot.slot_id.clone(),
        target_path: None,
        will_replace_target: false,
        metadata: slot.metadata.clone().normalized(),
        entity_count: slot.scene.entities.len(),
        resource_count: slot.scene.resources.len(),
    })
}

pub(in crate::scene::dynamic_scene::session) fn preview_selected_single_slot_archive(
    archive: &RuntimeSessionArchive,
    selector: RuntimeSessionSlotSelector,
) -> Result<RuntimeSessionSlotExportPreviewReport, RuntimeSessionArchiveError> {
    let report = archive.select_slot(selector)?;
    preview_single_slot_archive(archive, &report.selected_slot_id)
}

pub(in crate::scene::dynamic_scene::session) fn preview_single_slot_archive_to_path(
    archive: &RuntimeSessionArchive,
    slot_id: &str,
    target_path: impl AsRef<Path>,
) -> Result<RuntimeSessionSlotExportPreviewReport, RuntimeSessionArchiveError> {
    let target_path = target_path.as_ref();
    let mut report = preview_single_slot_archive(archive, slot_id)?;
    report.will_replace_target = target_path::target_file_will_replace(
        target_path,
        "runtime session single-slot archive target",
    )?;
    report.target_path = Some(target_path.to_path_buf());
    Ok(report)
}
