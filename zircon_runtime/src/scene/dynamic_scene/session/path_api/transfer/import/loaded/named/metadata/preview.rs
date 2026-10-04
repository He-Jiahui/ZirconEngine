use std::path::Path;

use super::super::super::super::super::super::super::{
    path_transfer, RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionMetadata,
    RuntimeSessionSlotImportPreviewReport,
};

impl RuntimeSessionArchive {
    /// 载入已有目标档案，预检来源档案中指定 ID 的槽位；新 ID 修剪后须非空且未被目标档案占用。
    /// 完整采用并规范化传入元数据。报告仅描述当前检查结果，不保留后续提交计划。
    pub fn preview_import_slot_from_archive_with_metadata_at_path(
        path: impl AsRef<Path>,
        incoming: &RuntimeSessionArchive,
        source_slot_id: &str,
        new_slot_id: impl Into<String>,
        metadata: RuntimeSessionMetadata,
    ) -> Result<RuntimeSessionSlotImportPreviewReport, RuntimeSessionArchiveError> {
        path_transfer::preview_import_slot_from_archive_with_metadata_at_path(
            path,
            incoming,
            source_slot_id,
            new_slot_id,
            metadata,
        )
    }
}
