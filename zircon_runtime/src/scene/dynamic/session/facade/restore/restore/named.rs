use crate::scene::{LevelSystem, World};

use super::super::super::super::{
    restore as session_restore, RuntimeSessionArchive, RuntimeSessionArchiveError,
    RuntimeSessionLevelRestoreReport,
};

impl RuntimeSessionArchive {
    /// 从槽位构造独立世界；适合先完成恢复，再决定是否替换当前关卡。
    pub fn restore_slot_to_empty_world(
        &self,
        slot_id: &str,
    ) -> Result<World, RuntimeSessionArchiveError> {
        session_restore::restore_slot_to_empty_world(self, slot_id)
    }

    /// 先构造槽位世界，再替换关卡并重置运行时状态；构造失败时保留旧世界。
    pub fn restore_slot_into_level(
        &self,
        slot_id: &str,
        level: &LevelSystem,
    ) -> Result<RuntimeSessionLevelRestoreReport, RuntimeSessionArchiveError> {
        session_restore::restore_slot_into_level(self, slot_id, level)
    }
}
