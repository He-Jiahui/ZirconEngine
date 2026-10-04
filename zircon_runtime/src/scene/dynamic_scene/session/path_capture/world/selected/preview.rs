use std::path::Path;

use crate::scene::World;

use super::super::super::super::{
    io, RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionMetadata,
    RuntimeSessionSlotCapturePreviewReport, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    /// 从本次加载的档案中解析选择器，返回命中槽位的 World 捕获预览；不写文件，提交会重新加载并解析。
    pub fn preview_capture_world_selected_slot_to_path(
        path: impl AsRef<Path>,
        selector: RuntimeSessionSlotSelector,
        world: &World,
        metadata: RuntimeSessionMetadata,
    ) -> Result<RuntimeSessionSlotCapturePreviewReport, RuntimeSessionArchiveError> {
        io::load_or_empty_from_path(path)?
            .preview_capture_world_selected_slot(selector, world, metadata)
    }

    /// 从本次加载的档案中解析选择器，返回保留命中槽位元数据的只读捕获预览；提交会重新加载并解析。
    pub fn preview_capture_world_selected_slot_preserving_metadata_to_path(
        path: impl AsRef<Path>,
        selector: RuntimeSessionSlotSelector,
        world: &World,
    ) -> Result<RuntimeSessionSlotCapturePreviewReport, RuntimeSessionArchiveError> {
        io::load_or_empty_from_path(path)?
            .preview_capture_world_selected_slot_preserving_metadata(selector, world)
    }
}
