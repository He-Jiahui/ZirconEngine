use std::path::Path;

use crate::scene::World;

use super::super::super::super::{
    io, RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionMetadata,
    RuntimeSessionSlotCapturePreviewReport,
};

impl RuntimeSessionArchive {
    /// 从路径读取（缺失时视为空档案）并生成 World 槽位捕获预览；不创建或写入目标文件。
    pub fn preview_capture_world_slot_to_path(
        path: impl AsRef<Path>,
        slot_id: impl Into<String>,
        world: &World,
        metadata: RuntimeSessionMetadata,
    ) -> Result<RuntimeSessionSlotCapturePreviewReport, RuntimeSessionArchiveError> {
        io::load_or_empty_from_path(path)?.preview_capture_world_slot(slot_id, world, metadata)
    }
}
