use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotSelectionReport,
    RuntimeSessionSlotSelector,
};

pub(in crate::scene::dynamic_scene::session) fn select_slot(
    archive: &RuntimeSessionArchive,
    selector: RuntimeSessionSlotSelector,
) -> Result<RuntimeSessionSlotSelectionReport, RuntimeSessionArchiveError> {
    selector.resolve(archive)
}
