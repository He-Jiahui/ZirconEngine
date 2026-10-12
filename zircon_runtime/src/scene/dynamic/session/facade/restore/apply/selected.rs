use crate::scene::{LevelSystem, World};

use super::super::super::super::super::EntityRemap;
use super::super::super::super::{
    restore as session_restore, RuntimeSessionArchive, RuntimeSessionArchiveError,
    RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    /// 先解析选择器，再合入现有世界；选择失败时尚未修改目标，成功返回实体映射。
    pub fn apply_selected_slot(
        &self,
        selector: RuntimeSessionSlotSelector,
        world: &mut World,
    ) -> Result<EntityRemap, RuntimeSessionArchiveError> {
        session_restore::apply_selected_slot(self, selector, world)
    }

    /// 先解析选择器，再合入关卡现有世界；保留已有实体并返回源到目标的映射。
    pub fn apply_selected_slot_to_level(
        &self,
        selector: RuntimeSessionSlotSelector,
        level: &LevelSystem,
    ) -> Result<EntityRemap, RuntimeSessionArchiveError> {
        session_restore::apply_selected_slot_to_level(self, selector, level)
    }
}
