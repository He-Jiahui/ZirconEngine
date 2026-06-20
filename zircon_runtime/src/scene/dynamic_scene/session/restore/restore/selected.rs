use crate::scene::{LevelSystem, World};

use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionLevelRestoreReport,
    RuntimeSessionSlotSelector,
};
use super::{restore_slot_into_level, restore_slot_to_empty_world};

pub(in crate::scene::dynamic_scene::session) fn restore_selected_slot_to_empty_world(
    archive: &RuntimeSessionArchive,
    selector: RuntimeSessionSlotSelector,
) -> Result<World, RuntimeSessionArchiveError> {
    let report = archive.select_slot(selector)?;
    restore_slot_to_empty_world(archive, &report.selected_slot_id)
}

pub(in crate::scene::dynamic_scene::session) fn restore_selected_slot_into_level(
    archive: &RuntimeSessionArchive,
    selector: RuntimeSessionSlotSelector,
    level: &LevelSystem,
) -> Result<RuntimeSessionLevelRestoreReport, RuntimeSessionArchiveError> {
    let report = archive.select_slot(selector)?;
    restore_slot_into_level(archive, &report.selected_slot_id, level)
}
