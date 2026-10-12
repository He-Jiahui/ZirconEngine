use super::super::slot_id::validate_canonical_slot_id;
use super::super::{RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlot};

// 追加外部构造的槽位：ID 必须已经规范化，元数据在此规范化，场景必须有效；同名槽位一律拒绝。
// 检查全部成功后才进入档案写入边界，供复制和导入保证目标原槽位不会被覆盖。
pub(in crate::scene::dynamic_scene::session) fn push_slot(
    archive: &mut RuntimeSessionArchive,
    mut slot: RuntimeSessionSlot,
) -> Result<(), RuntimeSessionArchiveError> {
    validate_canonical_slot_id(&slot.slot_id)?;
    slot.metadata.normalize();
    slot.scene.ensure_supported()?;
    if archive.indexed_slot(&slot.slot_id).is_some() {
        return Err(RuntimeSessionArchiveError::DuplicateSlotId {
            slot_id: slot.slot_id,
        });
    }
    archive.commit_slot_upsert(slot);
    Ok(())
}
