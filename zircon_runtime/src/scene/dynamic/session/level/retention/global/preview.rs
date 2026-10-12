use crate::scene::LevelSystem;

use super::super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError,
    RuntimeSessionArchiveRetentionPolicy, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    /// 只读预览关卡快照对当前选择器命中槽位的替换及保留策略结果。
    /// 采用关卡项目、资产和显示信息；旧槽位的会话标签及时间不会自动继承。
    /// 本次捕获槽位受保护；保护项优先于数量上限，实际保留数可超过该上限。
    /// 结果是当次快照摘要；后续提交会重新解析选择器，并不会消费或锁定此报告。
    pub fn preview_capture_level_selected_slot_with_retention(
        &self,
        selector: RuntimeSessionSlotSelector,
        level: &LevelSystem,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError> {
        let report = self.select_slot(selector)?;
        self.preview_capture_level_slot_with_retention(report.selected_slot_id, level, policy)
    }

    /// 只读预览关卡快照对当前选择器命中槽位的替换及保留策略结果。
    /// 沿用命中槽位的会话元数据，含标签和更新时间；捕获本身不会刷新这些字段。
    /// 本次捕获槽位受保护；保护项优先于数量上限，实际保留数可超过该上限。
    /// 结果是当次快照摘要；后续提交会重新解析选择器，并不会消费或锁定此报告。
    pub fn preview_capture_level_selected_slot_preserving_metadata_with_retention(
        &self,
        selector: RuntimeSessionSlotSelector,
        level: &LevelSystem,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError> {
        let report = self.select_slot(selector)?;
        let world = level.snapshot();
        self.preview_capture_world_slot_with_retention(
            report.selected_slot_id,
            &world,
            report.summary.metadata,
            policy,
        )
    }
}
