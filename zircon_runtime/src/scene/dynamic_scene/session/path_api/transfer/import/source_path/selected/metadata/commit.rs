use std::path::Path;

use super::super::super::super::super::super::super::{
    path_transfer, RuntimeSessionArchive, RuntimeSessionArchiveError,
    RuntimeSessionArchiveManifest, RuntimeSessionMetadata, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    /// 读取磁盘来源后向已有目标导入单槽；路径相等或双方规范路径相等时拒绝。
    /// 新 ID 修剪后须非空且未占用。
    /// 选择器在来源快照中解析；完整采用并规范化替换元数据。
    /// 成功返回表示目标替换已发布；原子发布不保证整个读改写流程免于并发覆盖。
    pub fn import_selected_slot_from_archive_path_with_metadata_at_path_atomically(
        path: impl AsRef<Path>,
        source_path: impl AsRef<Path>,
        selector: RuntimeSessionSlotSelector,
        new_slot_id: impl Into<String>,
        metadata: RuntimeSessionMetadata,
    ) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
        path_transfer::import_selected_slot_from_archive_path_with_metadata_at_path_atomically(
            path,
            source_path,
            selector,
            new_slot_id,
            metadata,
        )
    }
}
