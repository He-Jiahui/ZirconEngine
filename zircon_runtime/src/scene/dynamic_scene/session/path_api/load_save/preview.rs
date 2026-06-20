use std::path::Path;

use super::super::super::{
    io, RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchiveSavePreviewReport,
};

impl RuntimeSessionArchive {
    pub fn preview_save_to_path(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<RuntimeSessionArchiveSavePreviewReport, RuntimeSessionArchiveError> {
        io::preview_save_to_path(self, path)
    }
}
