use super::super::slot_id::validate_canonical_slot_id;
use super::super::{RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlot};

// 捕获等明确表达替换意图的入口使用更新插入；同名时整个槽位连同场景及元数据一起替换。
// 接收的 ID 必须已规范化，场景校验成功后才由档案边界同步索引和代际。
pub(in crate::scene::dynamic_scene::session) fn upsert_slot(
    archive: &mut RuntimeSessionArchive,
    mut slot: RuntimeSessionSlot,
) -> Result<(), RuntimeSessionArchiveError> {
    validate_canonical_slot_id(&slot.slot_id)?;
    slot.metadata.normalize();
    slot.scene.ensure_supported()?;
    archive.commit_slot_upsert(slot);
    Ok(())
}
