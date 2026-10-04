use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionMetadata,
};
use super::preview::preview_update_slot_metadata;

// 把预检后的元数据交给档案写入边界，确保时间和标签的派生索引随元数据一起更新；场景负载保留。
pub(in crate::scene::dynamic_scene::session) fn update_slot_metadata(
    archive: &mut RuntimeSessionArchive,
    slot_id: &str,
    metadata: RuntimeSessionMetadata,
) -> Result<(), RuntimeSessionArchiveError> {
    let report = preview_update_slot_metadata(archive, slot_id, metadata)?;
    if !archive.replace_slot_metadata(&report.source_slot_id, report.metadata) {
        return Err(RuntimeSessionArchiveError::MissingSlot {
            slot_id: report.source_slot_id,
        });
    }
    Ok(())
}
