use std::path::Path;

use super::super::super::super::super::{
    path_export, RuntimeSessionArchive, RuntimeSessionArchiveError,
    RuntimeSessionSlotExportPreviewReport,
};

impl RuntimeSessionArchive {
    /// 读取源归档并预检单槽另存目标；路径相等或已有路径的规范化结果相等时拒绝。
    /// 报告只反映当次目标状态，不预留目标写入。
    pub fn preview_save_single_slot_archive_from_path(
        source_path: impl AsRef<Path>,
        slot_id: &str,
        target_path: impl AsRef<Path>,
    ) -> Result<RuntimeSessionSlotExportPreviewReport, RuntimeSessionArchiveError> {
        path_export::preview_save_single_slot_archive_from_path(source_path, slot_id, target_path)
    }
}
