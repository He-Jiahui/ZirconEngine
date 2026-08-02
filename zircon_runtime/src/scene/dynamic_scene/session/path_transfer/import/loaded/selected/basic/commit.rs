use std::path::Path;

use super::super::super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchiveManifest,
    RuntimeSessionSlotSelector, io,
};

pub(in crate::scene::dynamic_scene::session) fn import_selected_slot_from_archive_at_path_atomically(
    path: impl AsRef<Path>,
    incoming: &RuntimeSessionArchive,
    selector: RuntimeSessionSlotSelector,
    new_slot_id: impl Into<String>,
) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
    io::mutate_archive_at_path_atomically(path, |archive| {
        archive.import_selected_slot_from_archive(incoming, selector, new_slot_id)?;
        Ok(())
    })
}
