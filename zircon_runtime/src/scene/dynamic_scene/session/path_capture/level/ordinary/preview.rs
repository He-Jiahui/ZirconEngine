use std::path::Path;

use crate::scene::LevelSystem;

use super::super::super::super::{
    io, RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotCapturePreviewReport,
};

impl RuntimeSessionArchive {
    /// 从路径读取（缺失时视为空档案）并生成 Level 槽位捕获预览；不创建或写入目标文件。
    pub fn preview_capture_level_slot_to_path(
        path: impl AsRef<Path>,
        slot_id: impl Into<String>,
        level: &LevelSystem,
    ) -> Result<RuntimeSessionSlotCapturePreviewReport, RuntimeSessionArchiveError> {
        io::load_or_empty_from_path(path)?.preview_capture_level_slot(slot_id, level)
    }
}
