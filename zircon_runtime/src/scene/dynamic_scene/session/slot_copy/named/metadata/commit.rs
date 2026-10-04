use super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionMetadata,
};
use super::preview::preview_copy_slot_with_metadata;

// 预检通过后从当前档案复制场景到新槽位，整体采用替换元数据；源槽位保持原样。
// 追加入口再次执行槽位有效性和重复 ID 检查，再统一更新目标索引与代际。
pub(in crate::scene::dynamic_scene::session) fn copy_slot_with_metadata(
    archive: &mut RuntimeSessionArchive,
    source_slot_id: &str,
    new_slot_id: impl Into<String>,
    metadata: RuntimeSessionMetadata,
) -> Result<(), RuntimeSessionArchiveError> {
    let report = preview_copy_slot_with_metadata(archive, source_slot_id, new_slot_id, metadata)?;
    let mut slot = archive.require_slot(source_slot_id)?.clone();
    slot.slot_id = report.destination_slot_id;
    slot.metadata = report.metadata;
    archive.push_slot(slot)
}
