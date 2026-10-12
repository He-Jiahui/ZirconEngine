use crate::scene::LevelSystem;

use super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    /// 在内存档案中提交关卡快照对当前选择器命中槽位的替换。
    /// 采用关卡项目、资产和显示信息；旧槽位的会话标签及时间不会自动继承。
    pub fn capture_level_selected_slot(
        &mut self,
        selector: RuntimeSessionSlotSelector,
        level: &LevelSystem,
    ) -> Result<(), RuntimeSessionArchiveError> {
        let report = self.select_slot(selector)?;
        self.capture_level_slot(report.selected_slot_id, level)
    }

    /// 在内存档案中提交关卡快照对当前选择器命中槽位的替换。
    /// 沿用命中槽位的会话元数据，含标签和更新时间；捕获本身不会刷新这些字段。
    pub fn capture_level_selected_slot_preserving_metadata(
        &mut self,
        selector: RuntimeSessionSlotSelector,
        level: &LevelSystem,
    ) -> Result<(), RuntimeSessionArchiveError> {
        let report = self.select_slot(selector)?;
        let world = level.snapshot();
        self.capture_world_slot(report.selected_slot_id, &world, report.summary.metadata)
    }
}
