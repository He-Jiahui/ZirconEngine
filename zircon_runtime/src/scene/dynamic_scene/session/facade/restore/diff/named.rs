use crate::scene::{LevelSystem, World};

use super::super::super::super::{
    restore as session_restore, RuntimeSessionArchive, RuntimeSessionArchiveError,
    RuntimeSessionSlotDiffReport,
};

impl RuntimeSessionArchive {
    /// 比较槽位场景与目标世界的捕获值是否结构相等；不修改目标。
    pub fn diff_slot_with_world(
        &self,
        slot_id: &str,
        world: &World,
    ) -> Result<RuntimeSessionSlotDiffReport, RuntimeSessionArchiveError> {
        session_restore::diff_slot_with_world(self, slot_id, world)
    }

    /// 比较槽位场景与关卡世界快照；关卡显示名等元数据不参与比较。
    pub fn diff_slot_with_level(
        &self,
        slot_id: &str,
        level: &LevelSystem,
    ) -> Result<RuntimeSessionSlotDiffReport, RuntimeSessionArchiveError> {
        session_restore::diff_slot_with_level(self, slot_id, level)
    }
}
