use super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotImportPreviewReport,
    RuntimeSessionSlotSelector,
};
use super::super::super::named::preview_import_slot_from_archive;

// 只读预览跨档案导入，选择器只在源档案解析；新 ID 属于目标档案且不得已存在。
// 继承源槽位的全部元数据，包括标签和更新时间。预览报告不锁定后续选择结果。
pub(in crate::scene::dynamic_scene::session) fn preview_import_selected_slot_from_archive(
    target: &RuntimeSessionArchive,
    incoming: &RuntimeSessionArchive,
    selector: RuntimeSessionSlotSelector,
    new_slot_id: impl Into<String>,
) -> Result<RuntimeSessionSlotImportPreviewReport, RuntimeSessionArchiveError> {
    let report = incoming.select_slot(selector)?;
    preview_import_slot_from_archive(target, incoming, &report.selected_slot_id, new_slot_id)
}
