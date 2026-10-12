use super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotSelector,
};
use super::super::super::named::import_slot_from_archive;

// 提交跨档案导入，选择器只在源档案解析；新 ID 属于目标档案且不得已存在。
// 继承源槽位的全部元数据，包括标签和更新时间。保留源槽位，本层仅修改目标内存档案。
pub(in crate::scene::dynamic_scene::session) fn import_selected_slot_from_archive(
    target: &mut RuntimeSessionArchive,
    incoming: &RuntimeSessionArchive,
    selector: RuntimeSessionSlotSelector,
    new_slot_id: impl Into<String>,
) -> Result<(), RuntimeSessionArchiveError> {
    let report = incoming.select_slot(selector)?;
    import_slot_from_archive(target, incoming, &report.selected_slot_id, new_slot_id)
}
