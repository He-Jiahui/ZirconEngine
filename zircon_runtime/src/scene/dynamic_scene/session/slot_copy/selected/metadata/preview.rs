use super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionMetadata,
    RuntimeSessionSlotImportPreviewReport, RuntimeSessionSlotSelector,
};
use super::super::super::named::preview_copy_slot_with_metadata;

// 只读预览同档案复制，选择器只在当前档案解析；新 ID 属于目标档案且不得已存在。
// 完整采用调用者的替换元数据。预览报告不锁定后续选择结果。
pub(in crate::scene::dynamic_scene::session) fn preview_copy_selected_slot_with_metadata(
    archive: &RuntimeSessionArchive,
    selector: RuntimeSessionSlotSelector,
    new_slot_id: impl Into<String>,
    metadata: RuntimeSessionMetadata,
) -> Result<RuntimeSessionSlotImportPreviewReport, RuntimeSessionArchiveError> {
    let report = archive.select_slot(selector)?;
    preview_copy_slot_with_metadata(archive, &report.selected_slot_id, new_slot_id, metadata)
}
