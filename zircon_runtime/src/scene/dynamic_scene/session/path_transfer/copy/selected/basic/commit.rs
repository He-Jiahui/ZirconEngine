use std::path::Path;

use super::super::super::super::super::{
    io, RuntimeSessionArchiveError, RuntimeSessionArchiveManifest, RuntimeSessionSlotSelector,
};

pub(in crate::scene::dynamic_scene::session) fn copy_selected_slot_at_path_atomically(
    path: impl AsRef<Path>,
    selector: RuntimeSessionSlotSelector,
    new_slot_id: impl Into<String>,
) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
    io::mutate_archive_at_path_atomically(path, |archive| {
        archive.copy_selected_slot(selector, new_slot_id)?;
        Ok(())
    })
}
