use crate::scene::World;

use super::super::super::super::super::super::*;

impl RuntimeSessionArchive {
    /// 预览世界捕获及指定标签桶留存；捕获槽位受保护，后续提交会重新捕获。
    pub fn preview_capture_world_slot_with_tag_retention(
        &self,
        tag: &str,
        slot_id: impl Into<String>,
        world: &World,
        metadata: RuntimeSessionMetadata,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError> {
        capture_retention::preview_world_slot_with_tag_retention(
            self, tag, slot_id, world, metadata, policy,
        )
    }
}
