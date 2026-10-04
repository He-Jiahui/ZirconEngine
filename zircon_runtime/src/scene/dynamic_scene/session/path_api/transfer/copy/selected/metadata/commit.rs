use std::path::Path;

use super::super::super::super::super::super::{
    path_transfer, RuntimeSessionArchive, RuntimeSessionArchiveError,
    RuntimeSessionArchiveManifest, RuntimeSessionMetadata, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    /// 在本次载入的档案上解析选择器后复制到新 ID，整体采用规范化后的替换元数据；保留源槽位。
    /// 新 ID 修剪后须非空且未占用。
    /// 保存成功才返回目录摘要；原子替换不保证整个读改写区间隔离。
    pub fn copy_selected_slot_with_metadata_at_path_atomically(
        path: impl AsRef<Path>,
        selector: RuntimeSessionSlotSelector,
        new_slot_id: impl Into<String>,
        metadata: RuntimeSessionMetadata,
    ) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
        path_transfer::copy_selected_slot_with_metadata_at_path_atomically(
            path,
            selector,
            new_slot_id,
            metadata,
        )
    }
}
