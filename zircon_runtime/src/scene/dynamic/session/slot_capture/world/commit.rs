use crate::scene::World;

use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionMetadata,
};
use super::preview::preview_world_slot;

// 命名捕获的内存提交边界：先得到完整的有效快照，再以相同 ID 更新插入，允许替换已有槽位。
// 提交沿用预检已捕获的槽位，不重新读取世界或关卡；预检失败时档案保持原样。
pub(in crate::scene::dynamic_scene::session) fn capture_world_slot(
    archive: &mut RuntimeSessionArchive,
    slot_id: impl Into<String>,
    world: &World,
    metadata: RuntimeSessionMetadata,
) -> Result<(), RuntimeSessionArchiveError> {
    let preview = preview_world_slot(archive, slot_id, world, metadata)?;
    archive.upsert_slot(preview.slot)
}
