use crate::scene::LevelSystem;

use super::super::super::super::super::*;

impl RuntimeSessionArchive {
    /// 捕获 Level 的世界及可持久化元数据到命名槽位；后续 Level 更新不会改变该快照。
    pub fn capture_level_slot(
        &mut self,
        slot_id: impl Into<String>,
        level: &LevelSystem,
    ) -> Result<(), RuntimeSessionArchiveError> {
        slot_capture::capture_level_slot(self, slot_id, level)
    }
}
