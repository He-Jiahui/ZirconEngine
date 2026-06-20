use crate::scene::{LevelSystem, World};

use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotDiffReport,
    RuntimeSessionSlotSelector,
};
use super::{diff_slot_with_level, diff_slot_with_world};

pub(in crate::scene::dynamic_scene::session) fn diff_selected_slot_with_world(
    archive: &RuntimeSessionArchive,
    selector: RuntimeSessionSlotSelector,
    world: &World,
) -> Result<RuntimeSessionSlotDiffReport, RuntimeSessionArchiveError> {
    let report = archive.select_slot(selector)?;
    diff_slot_with_world(archive, &report.selected_slot_id, world)
}

pub(in crate::scene::dynamic_scene::session) fn diff_selected_slot_with_level(
    archive: &RuntimeSessionArchive,
    selector: RuntimeSessionSlotSelector,
    level: &LevelSystem,
) -> Result<RuntimeSessionSlotDiffReport, RuntimeSessionArchiveError> {
    let report = archive.select_slot(selector)?;
    diff_slot_with_level(archive, &report.selected_slot_id, level)
}
