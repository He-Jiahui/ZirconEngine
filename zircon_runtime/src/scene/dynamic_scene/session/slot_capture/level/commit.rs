use crate::scene::LevelSystem;

use super::super::super::{RuntimeSessionArchive, RuntimeSessionArchiveError};
use super::preview::preview_level_slot;

pub(in crate::scene::dynamic_scene::session) fn capture_level_slot(
    archive: &mut RuntimeSessionArchive,
    slot_id: impl Into<String>,
    level: &LevelSystem,
) -> Result<(), RuntimeSessionArchiveError> {
    let preview = preview_level_slot(archive, slot_id, level)?;
    archive.upsert_slot(preview.slot)
}
