use crate::scene::World;

use super::super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError,
    RuntimeSessionArchiveRetentionPolicy, RuntimeSessionMetadata, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    /// 只读预览世界快照对当前选择器命中槽位的替换及保留策略结果。
    /// 使用调用者提供的元数据整体替换旧值；选择器必须命中已有槽位，此入口不负责创建新 ID。
    /// 本次捕获槽位受保护；标签仅限定裁剪范围，不限定选择器或自动添加标签。
    /// 结果是当次快照摘要；后续提交会重新解析选择器，并不会消费或锁定此报告。
    pub fn preview_capture_world_selected_slot_with_tag_retention(
        &self,
        tag: &str,
        selector: RuntimeSessionSlotSelector,
        world: &World,
        metadata: RuntimeSessionMetadata,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError> {
        let report = self.select_slot(selector)?;
        self.preview_capture_world_slot_with_tag_retention(
            tag,
            report.selected_slot_id,
            world,
            metadata,
            policy,
        )
    }

    /// 只读预览世界快照对当前选择器命中槽位的替换及保留策略结果。
    /// 沿用命中槽位的会话元数据，含标签和更新时间；捕获本身不会刷新这些字段。
    /// 本次捕获槽位受保护；标签仅限定裁剪范围，不限定选择器或自动添加标签。
    /// 结果是当次快照摘要；后续提交会重新解析选择器，并不会消费或锁定此报告。
    pub fn preview_capture_world_selected_slot_preserving_metadata_with_tag_retention(
        &self,
        tag: &str,
        selector: RuntimeSessionSlotSelector,
        world: &World,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError> {
        let report = self.select_slot(selector)?;
        self.preview_capture_world_slot_with_tag_retention(
            tag,
            report.selected_slot_id,
            world,
            report.summary.metadata,
            policy,
        )
    }
}
