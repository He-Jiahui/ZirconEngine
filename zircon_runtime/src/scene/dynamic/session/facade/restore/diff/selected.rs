use crate::scene::{LevelSystem, World};

use super::super::super::super::{
    restore as session_restore, RuntimeSessionArchive, RuntimeSessionArchiveError,
    RuntimeSessionSlotDiffReport, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    /// 按选择器解析实际槽位后比较世界快照；不修改目标世界。
    pub fn diff_selected_slot_with_world(
        &self,
        selector: RuntimeSessionSlotSelector,
        world: &World,
    ) -> Result<RuntimeSessionSlotDiffReport, RuntimeSessionArchiveError> {
        session_restore::diff_selected_slot_with_world(self, selector, world)
    }

    /// 按选择器解析实际槽位后比较关卡世界；关卡元数据不参与比较。
    pub fn diff_selected_slot_with_level(
        &self,
        selector: RuntimeSessionSlotSelector,
        level: &LevelSystem,
    ) -> Result<RuntimeSessionSlotDiffReport, RuntimeSessionArchiveError> {
        session_restore::diff_selected_slot_with_level(self, selector, level)
    }
}
