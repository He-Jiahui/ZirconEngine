use crate::scene::LevelSystem;

use super::super::super::super::super::*;

impl RuntimeSessionArchive {
    /// 只返回本次捕获摘要，不发布槽位；后续提交会重新捕获，不消费此报告。
    pub fn preview_capture_level_slot(
        &self,
        slot_id: impl Into<String>,
        level: &LevelSystem,
    ) -> Result<RuntimeSessionSlotCapturePreviewReport, RuntimeSessionArchiveError> {
        slot_capture::preview_level_slot(self, slot_id, level).map(|preview| preview.report)
    }
}
