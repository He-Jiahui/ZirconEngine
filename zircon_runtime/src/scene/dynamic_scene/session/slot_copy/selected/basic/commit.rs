use super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotSelector,
};
use super::super::super::named::copy_slot;

pub(in crate::scene::dynamic_scene::session) fn copy_selected_slot(
    archive: &mut RuntimeSessionArchive,
    selector: RuntimeSessionSlotSelector,
    new_slot_id: impl Into<String>,
) -> Result<(), RuntimeSessionArchiveError> {
    let report = archive.select_slot(selector)?;
    copy_slot(archive, &report.selected_slot_id, new_slot_id)
}
