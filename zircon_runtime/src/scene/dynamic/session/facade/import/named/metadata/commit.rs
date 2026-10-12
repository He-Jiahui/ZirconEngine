use super::super::super::super::super::super::{
    slot_import, RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionMetadata,
};

impl RuntimeSessionArchive {
    /// 导入来源档案中指定 ID 的槽位；新 ID 修剪后须非空且未被目标档案占用。
    /// 完整采用并规范化传入元数据。复制场景并追加目标槽位，来源保持原样。
    pub fn import_slot_from_archive_with_metadata(
        &mut self,
        incoming: &RuntimeSessionArchive,
        source_slot_id: &str,
        new_slot_id: impl Into<String>,
        metadata: RuntimeSessionMetadata,
    ) -> Result<(), RuntimeSessionArchiveError> {
        slot_import::import_slot_from_archive_with_metadata(
            self,
            incoming,
            source_slot_id,
            new_slot_id,
            metadata,
        )
    }
}
