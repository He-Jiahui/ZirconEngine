use crate::scene::LevelSystem;

use super::super::super::super::super::super::*;

impl RuntimeSessionArchive {
    /// 捕获关卡并提交指定标签桶的留存结果；本次捕获槽位受保护，失败不会发布部分档案更新。
    pub fn capture_level_slot_with_tag_retention(
        &mut self,
        tag: &str,
        slot_id: impl Into<String>,
        level: &LevelSystem,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError> {
        capture_retention::capture_level_slot_with_tag_retention(self, tag, slot_id, level, policy)
    }
}
