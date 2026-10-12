use std::path::Path;

use crate::scene::World;

use super::super::super::super::super::super::{
    io, RuntimeSessionArchive, RuntimeSessionArchiveCaptureRetentionReport,
    RuntimeSessionArchiveError, RuntimeSessionArchiveRetentionPolicy, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    /// 从路径加载档案后，捕获选择器命中的槽位并保留该槽位原有元数据，再计算指定标签桶保留结果，再通过原子保存发布完整档案。
    pub fn capture_world_selected_slot_preserving_metadata_with_tag_retention_to_path_atomically(
        path: impl AsRef<Path>,
        tag: &str,
        selector: RuntimeSessionSlotSelector,
        world: &World,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError> {
        let path = path.as_ref();
        let mut archive = io::load_or_empty_from_path(path)?;
        let report = archive.capture_world_selected_slot_preserving_metadata_with_tag_retention(
            tag, selector, world, policy,
        )?;
        io::save_to_path_atomically(&archive, path)?;
        Ok(report)
    }
}
