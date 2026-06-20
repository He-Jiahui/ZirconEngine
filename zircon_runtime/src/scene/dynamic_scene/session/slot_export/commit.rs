use super::super::{RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotSelector};
use super::preview::preview_single_slot_archive;

pub(in crate::scene::dynamic_scene::session) fn single_slot_archive(
    archive: &RuntimeSessionArchive,
    slot_id: &str,
) -> Result<RuntimeSessionArchive, RuntimeSessionArchiveError> {
    let report = preview_single_slot_archive(archive, slot_id)?;
    RuntimeSessionArchive::from_slots(vec![archive.require_slot(&report.source_slot_id)?.clone()])
}

pub(in crate::scene::dynamic_scene::session) fn selected_single_slot_archive(
    archive: &RuntimeSessionArchive,
    selector: RuntimeSessionSlotSelector,
) -> Result<RuntimeSessionArchive, RuntimeSessionArchiveError> {
    let report = archive.select_slot(selector)?;
    single_slot_archive(archive, &report.selected_slot_id)
}
