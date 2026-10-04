use crate::scene::LevelSystem;

use super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotCapturePreviewReport,
    RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    /// 只读预览关卡快照对当前选择器命中槽位的替换。
    /// 采用关卡项目、资产和显示信息；旧槽位的会话标签及时间不会自动继承。
    /// 结果是当次快照摘要；后续提交会重新解析选择器，并不会消费或锁定此报告。
    pub fn preview_capture_level_selected_slot(
        &self,
        selector: RuntimeSessionSlotSelector,
        level: &LevelSystem,
    ) -> Result<RuntimeSessionSlotCapturePreviewReport, RuntimeSessionArchiveError> {
        let report = self.select_slot(selector)?;
        self.preview_capture_level_slot(report.selected_slot_id, level)
    }

    /// 只读预览关卡快照对当前选择器命中槽位的替换。
    /// 沿用命中槽位的会话元数据，含标签和更新时间；捕获本身不会刷新这些字段。
    /// 结果是当次快照摘要；后续提交会重新解析选择器，并不会消费或锁定此报告。
    pub fn preview_capture_level_selected_slot_preserving_metadata(
        &self,
        selector: RuntimeSessionSlotSelector,
        level: &LevelSystem,
    ) -> Result<RuntimeSessionSlotCapturePreviewReport, RuntimeSessionArchiveError> {
        let report = self.select_slot(selector)?;
        let world = level.snapshot();
        self.preview_capture_world_slot(report.selected_slot_id, &world, report.summary.metadata)
    }
}
