use std::path::Path;

use super::{
    target_path, RuntimeSessionArchive, RuntimeSessionArchiveArtifact, RuntimeSessionArchiveError,
    RuntimeSessionArchiveSavePreviewReport,
};

// 预览也先封存完整档案，因此序列化或大小限制错误会在目标路径检查之前返回；封存不写文件。
pub(super) fn preview_save_to_path(
    archive: &RuntimeSessionArchive,
    target_path: impl AsRef<Path>,
) -> Result<RuntimeSessionArchiveSavePreviewReport, RuntimeSessionArchiveError> {
    let target_path = target_path.as_ref();
    preview_artifact_save_to_path(&archive.sealed_artifact()?, target_path)
}

pub(super) fn preview_artifact_save_to_path(
    artifact: &RuntimeSessionArchiveArtifact,
    target_path: impl AsRef<Path>,
) -> Result<RuntimeSessionArchiveSavePreviewReport, RuntimeSessionArchiveError> {
    let target_path = target_path.as_ref();

    Ok(RuntimeSessionArchiveSavePreviewReport {
        target_path: target_path.to_path_buf(),
        will_replace_target: target_path::target_file_will_replace(
            target_path,
            "runtime session archive target",
        )?,
        statistics: artifact.statistics().clone(),
    })
}
