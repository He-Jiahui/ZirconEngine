use std::path::Path;

use super::super::super::super::super::{
    io, RuntimeSessionArchiveError, RuntimeSessionArchiveManifest, RuntimeSessionMetadata,
    RuntimeSessionSlotSelector,
};

pub(in crate::scene::dynamic_scene::session) fn copy_selected_slot_with_metadata_at_path_atomically(
    path: impl AsRef<Path>,
    selector: RuntimeSessionSlotSelector,
    new_slot_id: impl Into<String>,
    metadata: RuntimeSessionMetadata,
) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
    io::mutate_archive_at_path_atomically(path, |archive| {
        archive.copy_selected_slot_with_metadata(selector, new_slot_id, metadata)?;
        Ok(())
    })
}
