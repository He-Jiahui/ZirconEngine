use crate::scene::LevelSystem;

use super::super::super::super::construction;
use super::super::super::super::*;

impl RuntimeSessionArchive {
    /// 在 Level 当前快照上建立单槽档案；后续 Level 改动不会回写该槽位。
    pub fn from_level(
        slot_id: impl Into<String>,
        level: &LevelSystem,
    ) -> Result<Self, RuntimeSessionArchiveError> {
        construction::from_level(slot_id, level)
    }
}
