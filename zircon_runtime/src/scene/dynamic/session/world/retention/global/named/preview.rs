use std::path::Path;

use crate::scene::World;

use super::super::super::super::super::super::{
    io, RuntimeSessionArchive, RuntimeSessionArchiveCaptureRetentionReport,
    RuntimeSessionArchiveError, RuntimeSessionArchiveRetentionPolicy, RuntimeSessionMetadata,
};

impl RuntimeSessionArchive {
    /// 从路径加载档案，捕获指定槽位并计算全档案保留结果后只返回报告，不写目标文件；后续提交会重新加载档案。
    pub fn preview_capture_world_slot_with_retention_to_path(
        path: impl AsRef<Path>,
        slot_id: impl Into<String>,
        world: &World,
        metadata: RuntimeSessionMetadata,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError> {
        io::load_or_empty_from_path(path)?
            .preview_capture_world_slot_with_retention(slot_id, world, metadata, policy)
    }
}
