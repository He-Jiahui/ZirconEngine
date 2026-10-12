use crate::scene::LevelSystem;

use super::super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError,
    RuntimeSessionArchiveRetentionPolicy, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    /// 在内存档案中提交关卡快照对当前选择器命中槽位的替换及保留策略结果。
    /// 采用关卡项目、资产和显示信息；旧槽位的会话标签及时间不会自动继承。
    /// 本次捕获槽位受保护；标签仅限定裁剪范围，不限定选择器或自动添加标签。
    pub fn capture_level_selected_slot_with_tag_retention(
        &mut self,
        tag: &str,
        selector: RuntimeSessionSlotSelector,
        level: &LevelSystem,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError> {
        let report = self.select_slot(selector)?;
        self.capture_level_slot_with_tag_retention(tag, report.selected_slot_id, level, policy)
    }

    /// 在内存档案中提交关卡快照对当前选择器命中槽位的替换及保留策略结果。
    /// 沿用命中槽位的会话元数据，含标签和更新时间；捕获本身不会刷新这些字段。
    /// 本次捕获槽位受保护；标签仅限定裁剪范围，不限定选择器或自动添加标签。
    pub fn capture_level_selected_slot_preserving_metadata_with_tag_retention(
        &mut self,
        tag: &str,
        selector: RuntimeSessionSlotSelector,
        level: &LevelSystem,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError> {
        let report = self.select_slot(selector)?;
        let world = level.snapshot();
        self.capture_world_slot_with_tag_retention(
            tag,
            report.selected_slot_id,
            &world,
            report.summary.metadata,
            policy,
        )
    }
}
