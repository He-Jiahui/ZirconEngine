use super::super::super::super::slot_id::normalize_slot_id;
use super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionMetadata,
    RuntimeSessionSlotImportPreviewReport,
};

// 跨档案导入的共同预检：确认档案有效、目标新 ID 空闲以及显式源 ID 存在。
// 返回采用规范化替换元数据的摘要；不复制场景、不写入档案，也不保留可供稍后提交的计划。
pub(in crate::scene::dynamic_scene::session) fn preview_import_slot_from_archive_with_metadata(
    target: &RuntimeSessionArchive,
    incoming: &RuntimeSessionArchive,
    source_slot_id: &str,
    new_slot_id: impl Into<String>,
    metadata: RuntimeSessionMetadata,
) -> Result<RuntimeSessionSlotImportPreviewReport, RuntimeSessionArchiveError> {
    target.ensure_supported()?;
    incoming.ensure_supported()?;

    let destination_slot_id = normalize_slot_id(new_slot_id.into())?;
    if target.contains_slot(&destination_slot_id) {
        return Err(RuntimeSessionArchiveError::DuplicateSlotId {
            slot_id: destination_slot_id,
        });
    }

    let slot =
        incoming
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
