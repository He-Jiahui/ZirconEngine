use super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotSelector,
};
use super::super::super::named::import_slot_from_archive;

pub(in crate::scene::dynamic_scene::session) fn import_selected_slot_from_archive(
    target: &mut RuntimeSessionArchive,
    incoming: &RuntimeSessionArchive,
    selector: RuntimeSessionSlotSelector,
    new_slot_id: impl Into<String>,
) -> Result<(), RuntimeSessionArchiveError> {
    let report = incoming.select_slot(selector)?;
    import_slot_from_archive(target, incoming, &report.selected_slot_id, new_slot_id)
}
