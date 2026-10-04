use crate::scene::{LevelSystem, World};

use super::super::super::super::{
    restore as session_restore, RuntimeSessionArchive, RuntimeSessionArchiveError,
    RuntimeSessionLevelRestoreReport, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    /// 按选择器解析实际槽位并构造独立世界；选择失败时不开始场景生成。
    pub fn restore_selected_slot_to_empty_world(
        &self,
        selector: RuntimeSessionSlotSelector,
    ) -> Result<World, RuntimeSessionArchiveError> {
        session_restore::restore_selected_slot_to_empty_world(self, selector)
    }

    /// 解析选择器并完成新世界构造后替换关卡；报告标识实际选中的槽位。
    pub fn restore_selected_slot_into_level(
        &self,
        selector: RuntimeSessionSlotSelector,
        level: &LevelSystem,
    ) -> Result<RuntimeSessionLevelRestoreReport, RuntimeSessionArchiveError> {
        session_restore::restore_selected_slot_into_level(self, selector, level)
    }
}
