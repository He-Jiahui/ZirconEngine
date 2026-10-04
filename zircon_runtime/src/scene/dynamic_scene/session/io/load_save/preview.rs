use std::path::Path;

use super::super::super::{
    archive_save, RuntimeSessionArchive, RuntimeSessionArchiveError,
    RuntimeSessionArchiveSavePreviewReport,
};

/// 获取当前代际的封存产物后检查保存目标；可能更新封存缓存和校验/序列化计数，不创建目录或写入文件。
pub(in crate::scene::dynamic_scene::session) fn preview_save_to_path(
    archive: &RuntimeSessionArchive,
    path: impl AsRef<Path>,
) -> Result<RuntimeSessionArchiveSavePreviewReport, RuntimeSessionArchiveError> {
    archive_save::preview_save_to_path(archive, path)
}
