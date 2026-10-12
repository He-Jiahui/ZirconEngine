use super::super::super::{RuntimeSessionArchive, RuntimeSessionArchiveError};
use super::preview::preview_touch_slot;

// 更新时间会改变后续最新或最旧选择的排序，必须通过元数据写入边界同时刷新索引；场景负载保留。
pub(in crate::scene::dynamic_scene::session) fn touch_slot(
    archive: &mut RuntimeSessionArchive,
    slot_id: &str,
    updated_at_unix_millis: u64,
) -> Result<(), RuntimeSessionArchiveError> {
    let report = preview_touch_slot(archive, slot_id, updated_at_unix_millis)?;
    if !archive.replace_slot_metadata(&report.source_slot_id, report.metadata) {
        return Err(RuntimeSessionArchiveError::MissingSlot {
            slot_id: report.source_slot_id,
        });
    }
    Ok(())
}
