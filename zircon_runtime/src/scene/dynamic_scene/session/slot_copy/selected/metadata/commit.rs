use super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionMetadata,
    RuntimeSessionSlotSelector,
};
use super::super::super::named::copy_slot_with_metadata;

// 提交同档案复制，选择器只在当前档案解析；新 ID 属于目标档案且不得已存在。
// 完整采用调用者的替换元数据。保留源槽位，本层仅修改目标内存档案。
pub(in crate::scene::dynamic_scene::session) fn copy_selected_slot_with_metadata(
    archive: &mut RuntimeSessionArchive,
    selector: RuntimeSessionSlotSelector,
    new_slot_id: impl Into<String>,
    metadata: RuntimeSessionMetadata,
) -> Result<(), RuntimeSessionArchiveError> {
    let report = archive.select_slot(selector)?;
    copy_slot_with_metadata(archive, &report.selected_slot_id, new_slot_id, metadata)
}
