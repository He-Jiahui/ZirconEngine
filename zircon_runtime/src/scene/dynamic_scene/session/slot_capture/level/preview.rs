use crate::scene::LevelSystem;

use super::super::super::{RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlot};
use super::super::preview::{capture_preview, RuntimeSessionSlotCapturePreview};

pub(in crate::scene::dynamic_scene::session) fn preview_level_slot(
    archive: &RuntimeSessionArchive,
    slot_id: impl Into<String>,
    level: &LevelSystem,
) -> Result<RuntimeSessionSlotCapturePreview, RuntimeSessionArchiveError> {
    archive.ensure_supported()?;
    capture_preview(archive, RuntimeSessionSlot::from_level(slot_id, level)?)
}
