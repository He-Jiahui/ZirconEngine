use crate::scene::World;

use super::super::super::super::super::super::*;

impl RuntimeSessionArchive {
    /// 将新快照与全局保留策略作为同一计划提交；新捕获槽位自动受保护，避免刚保存即被裁剪。
    pub fn capture_world_slot_with_retention(
        &mut self,
        slot_id: impl Into<String>,
        world: &World,
        metadata: RuntimeSessionMetadata,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError> {
        capture_retention::capture_world_slot_with_retention(self, slot_id, world, metadata, policy)
    }
}
