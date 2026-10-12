use super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionMetadata,
};
use super::preview::preview_import_slot_from_archive_with_metadata;

// 预检通过后从源档案复制场景到新槽位，整体采用替换元数据；源槽位保持原样。
// 追加入口再次执行槽位有效性和重复 ID 检查，再统一更新目标索引与代际。
pub(in crate::scene::dynamic_scene::session) fn import_slot_from_archive_with_metadata(
    target: &mut RuntimeSessionArchive,
    incoming: &RuntimeSessionArchive,
    source_slot_id: &str,
    new_slot_id: impl Into<String>,
    metadata: RuntimeSessionMetadata,
) -> Result<(), RuntimeSessionArchiveError> {
    let report = preview_import_slot_from_archive_with_metadata(
        target,
        incoming,
        source_slot_id,
        new_slot_id,
        metadata,
    )?;
    let mut slot = incoming.require_slot(source_slot_id)?.clone();
    slot.slot_id = report.destination_slot_id;
    slot.metadata = report.metadata;
    target.push_slot(slot)
}
