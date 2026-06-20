use std::path::Path;

use super::{
    target_path, RuntimeSessionArchive, RuntimeSessionArchiveError,
    RuntimeSessionArchiveSavePreviewReport,
};

pub(super) fn preview_save_to_path(
    archive: &RuntimeSessionArchive,
    target_path: impl AsRef<Path>,
) -> Result<RuntimeSessionArchiveSavePreviewReport, RuntimeSessionArchiveError> {
    let target_path = target_path.as_ref();
    let statistics = archive.statistics()?;

    Ok(RuntimeSessionArchiveSavePreviewReport {
        target_path: target_path.to_path_buf(),
        will_replace_target: target_path::target_file_will_replace(
            target_path,
            "runtime session archive target",
        )?,
        statistics,
    })
}
