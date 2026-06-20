use super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionMetadata,
    RuntimeSessionSlotSelector,
};
use super::super::super::named::import_slot_from_archive_with_metadata;

pub(in crate::scene::dynamic_scene::session) fn import_selected_slot_from_archive_with_metadata(
    target: &mut RuntimeSessionArchive,
    incoming: &RuntimeSessionArchive,
    selector: RuntimeSessionSlotSelector,
    new_slot_id: impl Into<String>,
    metadata: RuntimeSessionMetadata,
) -> Result<(), RuntimeSessionArchiveError> {
    let report = incoming.select_slot(selector)?;
    import_slot_from_archive_with_metadata(
        target,
        incoming,
        &report.selected_slot_id,
        new_slot_id,
        metadata,
    )
}
