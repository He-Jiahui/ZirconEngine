use super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotSelector,
};
use super::super::super::named::copy_slot;

// 提交同档案复制，选择器只在当前档案解析；新 ID 属于目标档案且不得已存在。
// 继承源槽位的全部元数据，包括标签和更新时间。保留源槽位，本层仅修改目标内存档案。
pub(in crate::scene::dynamic_scene::session) fn copy_selected_slot(
    archive: &mut RuntimeSessionArchive,
    selector: RuntimeSessionSlotSelector,
    new_slot_id: impl Into<String>,
) -> Result<(), RuntimeSessionArchiveError> {
    let report = archive.select_slot(selector)?;
    copy_slot(archive, &report.selected_slot_id, new_slot_id)
}
