use super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionMetadata,
    RuntimeSessionSlotImportPreviewReport, RuntimeSessionSlotSelector,
};
use super::super::super::named::preview_import_slot_from_archive_with_metadata;

// 只读预览跨档案导入，选择器只在源档案解析；新 ID 属于目标档案且不得已存在。
// 完整采用调用者的替换元数据。预览报告不锁定后续选择结果。
pub(in crate::scene::dynamic_scene::session) fn preview_import_selected_slot_from_archive_with_metadata(
    target: &RuntimeSessionArchive,
    incoming: &RuntimeSessionArchive,
    selector: RuntimeSessionSlotSelector,
    new_slot_id: impl Into<String>,
    metadata: RuntimeSessionMetadata,
) -> Result<RuntimeSessionSlotImportPreviewReport, RuntimeSessionArchiveError> {
    let report = incoming.select_slot(selector)?;
    preview_import_slot_from_archive_with_metadata(
        target,
        incoming,
        &report.selected_slot_id,
        new_slot_id,
        metadata,
    )
}
