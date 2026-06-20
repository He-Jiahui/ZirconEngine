use crate::scene::LevelSystem;

use super::super::super::super::construction;
use super::super::super::super::*;

impl RuntimeSessionArchive {
    pub fn from_level(
        slot_id: impl Into<String>,
        level: &LevelSystem,
    ) -> Result<Self, RuntimeSessionArchiveError> {
        construction::from_level(slot_id, level)
    }
}
