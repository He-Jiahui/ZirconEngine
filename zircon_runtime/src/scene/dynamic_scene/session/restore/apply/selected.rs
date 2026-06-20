use crate::scene::{EntityRemap, LevelSystem, World};

use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotSelector,
};
use super::{apply_slot, apply_slot_to_level};

pub(in crate::scene::dynamic_scene::session) fn apply_selected_slot(
    archive: &RuntimeSessionArchive,
    selector: RuntimeSessionSlotSelector,
    world: &mut World,
) -> Result<EntityRemap, RuntimeSessionArchiveError> {
    let report = archive.select_slot(selector)?;
    apply_slot(archive, &report.selected_slot_id, world)
}

pub(in crate::scene::dynamic_scene::session) fn apply_selected_slot_to_level(
    archive: &RuntimeSessionArchive,
    selector: RuntimeSessionSlotSelector,
    level: &LevelSystem,
) -> Result<EntityRemap, RuntimeSessionArchiveError> {
    let report = archive.select_slot(selector)?;
    apply_slot_to_level(archive, &report.selected_slot_id, level)
}
