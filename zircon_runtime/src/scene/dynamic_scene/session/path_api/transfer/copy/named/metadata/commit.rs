use std::path::Path;

use super::super::super::super::super::super::{
    path_transfer, RuntimeSessionArchive, RuntimeSessionArchiveError,
    RuntimeSessionArchiveManifest, RuntimeSessionMetadata,
};

impl RuntimeSessionArchive {
    /// 载入已有档案，按原值查找源槽位后复制到新 ID，整体采用规范化后的替换元数据；保留源槽位。
    /// 新 ID 修剪后须非空且未占用。
    /// 保存成功才返回目录摘要；原子替换不保证整个读改写区间隔离。
    pub fn copy_slot_with_metadata_at_path_atomically(
        path: impl AsRef<Path>,
        source_slot_id: &str,
        new_slot_id: impl Into<String>,
        metadata: RuntimeSessionMetadata,
    ) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
        path_transfer::copy_slot_with_metadata_at_path_atomically(
            path,
            source_slot_id,
            new_slot_id,
            metadata,
        )
    }
}
