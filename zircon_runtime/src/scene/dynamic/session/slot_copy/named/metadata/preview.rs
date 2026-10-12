use super::super::super::super::slot_id::normalize_slot_id;
use super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionMetadata,
    RuntimeSessionSlotImportPreviewReport,
};

// 同档案复制的共同预检：确认档案有效、目标新 ID 空闲以及显式源 ID 存在。
// 返回采用规范化替换元数据的摘要；不复制场景、不写入档案，也不保留可供稍后提交的计划。
pub(in crate::scene::dynamic_scene::session) fn preview_copy_slot_with_metadata(
    archive: &RuntimeSessionArchive,
    source_slot_id: &str,
    new_slot_id: impl Into<String>,
    metadata: RuntimeSessionMetadata,
) -> Result<RuntimeSessionSlotImportPreviewReport, RuntimeSessionArchiveError> {
    archive.ensure_supported()?;

    let destination_slot_id = normalize_slot_id(new_slot_id.into())?;
    if archive.contains_slot(&destination_slot_id) {
        return Err(RuntimeSessionArchiveError::DuplicateSlotId {
            slot_id: destination_slot_id,
        });
    }

    let slot =
        archive
            .slot(source_slot_id)
            .ok_or_else(|| RuntimeSessionArchiveError::MissingSlot {
                slot_id: source_slot_id.to_string(),
            })?;

    Ok(RuntimeSessionSlotImportPreviewReport {
        source_slot_id: slot.slot_id.clone(),
        destination_slot_id,
        metadata: metadata.normalized(),
        entity_count: slot.scene.entities.len(),
        resource_count: slot.scene.resources.len(),
    })
}
