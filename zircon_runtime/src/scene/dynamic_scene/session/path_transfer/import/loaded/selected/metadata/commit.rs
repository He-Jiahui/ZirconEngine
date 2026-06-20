use std::path::Path;

use super::super::super::super::super::super::{
    io, RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchiveManifest,
    RuntimeSessionMetadata, RuntimeSessionSlotSelector,
};

pub(in crate::scene::dynamic_scene::session) fn import_selected_slot_from_archive_with_metadata_at_path_atomically(
    path: impl AsRef<Path>,
    incoming: &RuntimeSessionArchive,
    selector: RuntimeSessionSlotSelector,
    new_slot_id: impl Into<String>,
    metadata: RuntimeSessionMetadata,
) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
    io::mutate_archive_at_path_atomically(path, |archive| {
        archive.import_selected_slot_from_archive_with_metadata(
            incoming,
            selector,
            new_slot_id,
            metadata,
        )?;
        Ok(())
    })
}
