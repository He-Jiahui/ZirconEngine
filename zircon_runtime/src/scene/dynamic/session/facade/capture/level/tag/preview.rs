use crate::scene::LevelSystem;

use super::super::super::super::super::super::*;

impl RuntimeSessionArchive {
    /// 预览关卡捕获及指定标签桶留存；捕获槽位受保护，后续提交会重新捕获。
    pub fn preview_capture_level_slot_with_tag_retention(
        &self,
        tag: &str,
        slot_id: impl Into<String>,
        level: &LevelSystem,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError> {
        capture_retention::preview_level_slot_with_tag_retention(self, tag, slot_id, level, policy)
    }
}
