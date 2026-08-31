use std::path::Path;

use super::super::super::super::super::{
    RuntimeSessionArchiveError, RuntimeSessionArchiveManifest, io,
};

pub(in crate::scene::dynamic_scene::session) fn copy_slot_at_path_atomically(
    path: impl AsRef<Path>,
    source_slot_id: &str,
    new_slot_id: impl Into<String>,
) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
    io::mutate_archive_at_path_atomically(path, |archive| {
        archive.copy_slot(source_slot_id, new_slot_id)?;
        Ok(())
    })
}
