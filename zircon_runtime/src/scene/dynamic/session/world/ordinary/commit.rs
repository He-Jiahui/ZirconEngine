use crate::scene::World;

use super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionMetadata,
    RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    /// 在内存档案中提交世界快照对当前选择器命中槽位的替换。
    /// 使用调用者提供的元数据整体替换旧值；选择器必须命中已有槽位，此入口不负责创建新 ID。
    pub fn capture_world_selected_slot(
        &mut self,
        selector: RuntimeSessionSlotSelector,
        world: &World,
        metadata: RuntimeSessionMetadata,
    ) -> Result<(), RuntimeSessionArchiveError> {
        let report = self.select_slot(selector)?;
        self.capture_world_slot(report.selected_slot_id, world, metadata)
    }

    /// 在内存档案中提交世界快照对当前选择器命中槽位的替换。
    /// 沿用命中槽位的会话元数据，含标签和更新时间；捕获本身不会刷新这些字段。
    pub fn capture_world_selected_slot_preserving_metadata(
        &mut self,
        selector: RuntimeSessionSlotSelector,
        world: &World,
    ) -> Result<(), RuntimeSessionArchiveError> {
        let report = self.select_slot(selector)?;
        self.capture_world_slot(report.selected_slot_id, world, report.summary.metadata)
    }
}
