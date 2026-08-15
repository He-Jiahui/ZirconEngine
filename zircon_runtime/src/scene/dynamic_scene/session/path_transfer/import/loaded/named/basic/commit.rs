use std::path::Path;

use super::super::super::super::super::super::{
    io, RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchiveManifest,
};

pub(in crate::scene::dynamic_scene::session) fn import_slot_from_archive_at_path_atomically(
    path: impl AsRef<Path>,
    incoming: &RuntimeSessionArchive,
    source_slot_id: &str,
    new_slot_id: impl Into<String>,
) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
    io::mutate_archive_at_path_atomically(path, |archive| {
        archive.import_slot_from_archive(incoming, source_slot_id, new_slot_id)?;
        Ok(())
    })
}
