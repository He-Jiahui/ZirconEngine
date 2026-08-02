use std::path::Path;

use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchiveSavePreviewReport,
    archive_save,
};

pub(in crate::scene::dynamic_scene::session) fn preview_save_to_path(
    archive: &RuntimeSessionArchive,
    path: impl AsRef<Path>,
) -> Result<RuntimeSessionArchiveSavePreviewReport, RuntimeSessionArchiveError> {
    archive_save::preview_save_to_path(archive, path)
}
