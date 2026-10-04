use std::path::Path;

use super::super::super::super::super::super::super::{
    path_transfer, RuntimeSessionArchive, RuntimeSessionArchiveError,
    RuntimeSessionArchiveManifest, RuntimeSessionMetadata, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    /// 导入来源档案中选择器解析的槽位；新 ID 修剪后须非空且未被目标档案占用。
    /// 完整采用并规范化传入元数据。目标路径须已存在；成功保存后返回目录摘要，读改写期间不提供跨调用事务隔离。
    pub fn import_selected_slot_from_archive_with_metadata_at_path_atomically(
        path: impl AsRef<Path>,
        incoming: &RuntimeSessionArchive,
        selector: RuntimeSessionSlotSelector,
        new_slot_id: impl Into<String>,
        metadata: RuntimeSessionMetadata,
    ) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
        path_transfer::import_selected_slot_from_archive_with_metadata_at_path_atomically(
            path,
            incoming,
            selector,
            new_slot_id,
            metadata,
        )
    }
}
