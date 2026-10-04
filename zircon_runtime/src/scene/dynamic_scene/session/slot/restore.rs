use crate::scene::{LevelSystem, World};

use super::super::super::EntityRemap;
use super::super::{RuntimeSessionArchiveError, RuntimeSessionLevelRestoreReport};
use super::RuntimeSessionSlot;

impl RuntimeSessionSlot {
    /// 将槽位内容增量生成到现有 World，并返回源实体到目标实体的映射；不会清空目标。
    pub fn apply_to_world(
        &self,
        world: &mut World,
    ) -> Result<EntityRemap, RuntimeSessionArchiveError> {
        Ok(self.scene.spawn_into(world)?)
    }

    /// 从空世界构造独立副本；用于完整 Level 替换前的失败隔离。
    pub fn restore_to_empty_world(&self) -> Result<World, RuntimeSessionArchiveError> {
        let mut world = World::empty();
        self.apply_to_world(&mut world)?;
        Ok(world)
    }

    /// 先完成场景构造，再替换 Level 世界并写入可映射的元数据；失败时旧世界仍可用。
    pub fn restore_into_level(
        &self,
        level: &LevelSystem,
    ) -> Result<RuntimeSessionLevelRestoreReport, RuntimeSessionArchiveError> {
        let world = self.restore_to_empty_world()?;
        let entity_count = world.node_records().len();
        let metadata = self.metadata.to_level_metadata();
        // TODO: [CR-DYNAMIC-SESSION-FACADE-0003] 确认并发读取是否要求世界与元数据同一时刻可见；当前两次独立加锁，中间可观察到新世界配旧元数据，缺少联合快照契约与并发测试。
        level.replace_world_and_reset_runtime_state(world);
        level.set_metadata(metadata.clone());
        Ok(RuntimeSessionLevelRestoreReport {
            slot_id: self.slot_id.clone(),
            metadata,
            entity_count,
        })
    }

    pub fn apply_to_level(
        &self,
        level: &LevelSystem,
    ) -> Result<EntityRemap, RuntimeSessionArchiveError> {
        level.with_world_mut(|world| self.apply_to_world(world))
    }
}
