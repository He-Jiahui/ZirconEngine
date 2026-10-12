use std::path::Path;

use crate::scene::LevelSystem;

use super::super::super::super::super::super::{
    io, RuntimeSessionArchive, RuntimeSessionArchiveCaptureRetentionReport,
    RuntimeSessionArchiveError, RuntimeSessionArchiveRetentionPolicy,
};

impl RuntimeSessionArchive {
    /// 从路径加载档案后，捕获指定槽位并计算全档案保留结果，再通过原子保存发布完整档案。
    pub fn capture_level_slot_with_retention_to_path_atomically(
        path: impl AsRef<Path>,
        slot_id: impl Into<String>,
        level: &LevelSystem,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError> {
        let path = path.as_ref();
        let mut archive = io::load_or_empty_from_path(path)?;
        let report = archive.capture_level_slot_with_retention(slot_id, level, policy)?;
        io::save_to_path_atomically(&archive, path)?;
        Ok(report)
    }
}
