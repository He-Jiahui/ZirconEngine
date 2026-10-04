use crate::scene::World;

use super::super::super::super::super::*;

impl RuntimeSessionArchive {
    /// 将 World 快照写入内存档案的命名槽位；可用于保存前组装，路径持久化需另行提交。
    pub fn capture_world_slot(
        &mut self,
        slot_id: impl Into<String>,
        world: &World,
        metadata: RuntimeSessionMetadata,
    ) -> Result<(), RuntimeSessionArchiveError> {
        slot_capture::capture_world_slot(self, slot_id, world, metadata)
    }
}
