use std::path::Path;

use crate::scene::LevelSystem;

use super::super::super::super::super::super::{
    io, RuntimeSessionArchive, RuntimeSessionArchiveCaptureRetentionReport,
    RuntimeSessionArchiveError, RuntimeSessionArchiveRetentionPolicy,
};

impl RuntimeSessionArchive {
    /// 从路径加载档案，捕获指定槽位并计算指定标签桶保留结果后只返回报告，不写目标文件；后续提交会重新加载档案。
    pub fn preview_capture_level_slot_with_tag_retention_to_path(
        path: impl AsRef<Path>,
        tag: &str,
        slot_id: impl Into<String>,
        level: &LevelSystem,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError> {
        io::load_or_empty_from_path(path)?
            .preview_capture_level_slot_with_tag_retention(tag, slot_id, level, policy)
    }
}
