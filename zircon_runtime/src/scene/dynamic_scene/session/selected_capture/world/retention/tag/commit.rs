use crate::scene::World;

use super::super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError,
    RuntimeSessionArchiveRetentionPolicy, RuntimeSessionMetadata, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    /// 在内存档案中提交世界快照对当前选择器命中槽位的替换及保留策略结果。
    /// 使用调用者提供的元数据整体替换旧值；选择器必须命中已有槽位，此入口不负责创建新 ID。
    /// 本次捕获槽位受保护；标签仅限定裁剪范围，不限定选择器或自动添加标签。
    pub fn capture_world_selected_slot_with_tag_retention(
        &mut self,
        tag: &str,
        selector: RuntimeSessionSlotSelector,
        world: &World,
        metadata: RuntimeSessionMetadata,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError> {
        let report = self.select_slot(selector)?;
        self.capture_world_slot_with_tag_retention(
            tag,
            report.selected_slot_id,
            world,
            metadata,
            policy,
        )
    }

    /// 在内存档案中提交世界快照对当前选择器命中槽位的替换及保留策略结果。
    /// 沿用命中槽位的会话元数据，含标签和更新时间；捕获本身不会刷新这些字段。
    /// 本次捕获槽位受保护；标签仅限定裁剪范围，不限定选择器或自动添加标签。
    pub fn capture_world_selected_slot_preserving_metadata_with_tag_retention(
        &mut self,
        tag: &str,
        selector: RuntimeSessionSlotSelector,
        world: &World,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError> {
        let report = self.select_slot(selector)?;
        self.capture_world_slot_with_tag_retention(
            tag,
            report.selected_slot_id,
            world,
            report.summary.metadata,
            policy,
        )
    }
}
