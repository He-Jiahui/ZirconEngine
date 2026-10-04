use super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionMetadata,
    RuntimeSessionSlotSelector,
};
use super::super::super::named::import_slot_from_archive_with_metadata;

// 提交跨档案导入，选择器只在源档案解析；新 ID 属于目标档案且不得已存在。
// 完整采用调用者的替换元数据。保留源槽位，本层仅修改目标内存档案。
pub(in crate::scene::dynamic_scene::session) fn import_selected_slot_from_archive_with_metadata(
    target: &mut RuntimeSessionArchive,
    incoming: &RuntimeSessionArchive,
    selector: RuntimeSessionSlotSelector,
    new_slot_id: impl Into<String>,
    metadata: RuntimeSessionMetadata,
) -> Result<(), RuntimeSessionArchiveError> {
    let report = incoming.select_slot(selector)?;
    import_slot_from_archive_with_metadata(
        target,
        incoming,
        &report.selected_slot_id,
        new_slot_id,
        metadata,
    )
}
