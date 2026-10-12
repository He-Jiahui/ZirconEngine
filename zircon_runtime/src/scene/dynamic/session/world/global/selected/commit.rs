use std::path::Path;

use crate::scene::World;

use super::super::super::super::super::super::{
    io, RuntimeSessionArchive, RuntimeSessionArchiveCaptureRetentionReport,
    RuntimeSessionArchiveError, RuntimeSessionArchiveRetentionPolicy, RuntimeSessionMetadata,
    RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    /// 每次从路径重载档案并重新解析选择器，捕获本次重新解析后命中的槽位并计算全档案保留结果，再通过原子保存发布完整档案。
    pub fn capture_world_selected_slot_with_retention_to_path_atomically(
        path: impl AsRef<Path>,
        selector: RuntimeSessionSlotSelector,
        world: &World,
        metadata: RuntimeSessionMetadata,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError> {
        let path = path.as_ref();
        let mut archive = io::load_or_empty_from_path(path)?;
        let report = archive
            .capture_world_selected_slot_with_retention(selector, world, metadata, policy)?;
        io::save_to_path_atomically(&archive, path)?;
        Ok(report)
    }
}
