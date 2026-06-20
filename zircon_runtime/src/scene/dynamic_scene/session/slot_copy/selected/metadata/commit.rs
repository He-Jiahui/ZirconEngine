use super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionMetadata,
    RuntimeSessionSlotSelector,
};
use super::super::super::named::copy_slot_with_metadata;

pub(in crate::scene::dynamic_scene::session) fn copy_selected_slot_with_metadata(
    archive: &mut RuntimeSessionArchive,
    selector: RuntimeSessionSlotSelector,
    new_slot_id: impl Into<String>,
    metadata: RuntimeSessionMetadata,
) -> Result<(), RuntimeSessionArchiveError> {
    let report = archive.select_slot(selector)?;
    copy_slot_with_metadata(archive, &report.selected_slot_id, new_slot_id, metadata)
}
