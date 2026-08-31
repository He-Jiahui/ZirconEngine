use std::path::Path;

use crate::scene::{LevelSystem, World};

use super::super::super::super::super::EntityRemap;
use super::super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotSelector, path_restore,
};

impl RuntimeSessionArchive {
    pub fn apply_selected_slot_from_path_to_world(
        path: impl AsRef<Path>,
        selector: RuntimeSessionSlotSelector,
        world: &mut World,
    ) -> Result<EntityRemap, RuntimeSessionArchiveError> {
        path_restore::apply_selected_slot_from_path_to_world(path, selector, world)
    }

    pub fn apply_selected_slot_from_path_to_level(
        path: impl AsRef<Path>,
        selector: RuntimeSessionSlotSelector,
        level: &LevelSystem,
    ) -> Result<EntityRemap, RuntimeSessionArchiveError> {
        path_restore::apply_selected_slot_from_path_to_level(path, selector, level)
    }
}
