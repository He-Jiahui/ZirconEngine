use crate::scene::{LevelSystem, World};

use super::super::super::super::super::EntityRemap;
use super::super::super::super::{
    restore as session_restore, RuntimeSessionArchive, RuntimeSessionArchiveError,
};

impl RuntimeSessionArchive {
    /// 将槽位合入现有世界并返回实体映射；已有实体保留，源 ID 冲突由映射处理。
    pub fn apply_slot(
        &self,
        slot_id: &str,
        world: &mut World,
    ) -> Result<EntityRemap, RuntimeSessionArchiveError> {
        session_restore::apply_slot(self, slot_id, world)
    }

    /// 将槽位合入关卡的现有世界；实体映射供后续引用使用，不替换关卡元数据。
    pub fn apply_slot_to_level(
        &self,
        slot_id: &str,
        level: &LevelSystem,
    ) -> Result<EntityRemap, RuntimeSessionArchiveError> {
        session_restore::apply_slot_to_level(self, slot_id, level)
    }
}
