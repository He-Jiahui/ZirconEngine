use super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotImportPreviewReport,
    RuntimeSessionSlotSelector,
};
use super::super::super::named::preview_copy_slot;

// 只读预览同档案复制，选择器只在当前档案解析；新 ID 属于目标档案且不得已存在。
// 继承源槽位的全部元数据，包括标签和更新时间。预览报告不锁定后续选择结果。
pub(in crate::scene::dynamic_scene::session) fn preview_copy_selected_slot(
    archive: &RuntimeSessionArchive,
    selector: RuntimeSessionSlotSelector,
    new_slot_id: impl Into<String>,
) -> Result<RuntimeSessionSlotImportPreviewReport, RuntimeSessionArchiveError> {
    let report = archive.select_slot(selector)?;
    preview_copy_slot(archive, &report.selected_slot_id, new_slot_id)
}
