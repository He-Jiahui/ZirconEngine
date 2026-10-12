use crate::scene::{LevelSystem, World};

use super::super::super::DynamicScene;
use super::super::{RuntimeSessionArchiveError, RuntimeSessionSlotDiffReport};
use super::RuntimeSessionSlot;

impl RuntimeSessionSlot {
    /// 将目标 World 捕获为动态场景后作整体比较；调用方据 matches 判断快照等价性。
    pub fn diff_world(
        &self,
        world: &World,
    ) -> Result<RuntimeSessionSlotDiffReport, RuntimeSessionArchiveError> {
        let target_scene = DynamicScene::from_world(world)?;
        Ok(RuntimeSessionSlotDiffReport {
            slot_id: self.slot_id.clone(),
            matches: self.scene == target_scene,
            slot_entity_count: self.scene.entities.len(),
            target_entity_count: target_scene.entities.len(),
            slot_resource_count: self.scene.resources.len(),
            target_resource_count: target_scene.resources.len(),
        })
    }

    /// 只比较 Level 的 World 快照与槽位场景；显示名和项目元数据不计入 matches。
    pub fn diff_level(
        &self,
        level: &LevelSystem,
    ) -> Result<RuntimeSessionSlotDiffReport, RuntimeSessionArchiveError> {
        self.diff_world(&level.snapshot())
    }
}
