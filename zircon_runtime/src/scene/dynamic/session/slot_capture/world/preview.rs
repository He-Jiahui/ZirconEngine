use crate::scene::World;

use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionMetadata, RuntimeSessionSlot,
};
use super::super::preview::{capture_preview, RuntimeSessionSlotCapturePreview};

// 先验证档案格式和场景快照，再把槽位与摘要配对；返回值仍是内存预检，调用者决定是否提交。
pub(in crate::scene::dynamic_scene::session) fn preview_world_slot(
    archive: &RuntimeSessionArchive,
    slot_id: impl Into<String>,
    world: &World,
    metadata: RuntimeSessionMetadata,
) -> Result<RuntimeSessionSlotCapturePreview, RuntimeSessionArchiveError> {
    archive.ensure_supported()?;
    capture_preview(
        archive,
        RuntimeSessionSlot::from_world_with_metadata(slot_id, world, metadata)?,
    )
}
