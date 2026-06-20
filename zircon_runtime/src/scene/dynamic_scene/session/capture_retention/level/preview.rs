use crate::scene::LevelSystem;

use super::super::super::{
    slot_capture, RuntimeSessionArchive, RuntimeSessionArchiveCaptureRetentionReport,
    RuntimeSessionArchiveError, RuntimeSessionArchiveRetentionPolicy,
};
use super::super::apply::apply_capture_preview_with_retention;

pub(in crate::scene::dynamic_scene::session) fn preview_level_slot_with_retention(
    archive: &RuntimeSessionArchive,
    slot_id: impl Into<String>,
    level: &LevelSystem,
    policy: RuntimeSessionArchiveRetentionPolicy,
) -> Result<RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError> {
    let preview = slot_capture::preview_level_slot(archive, slot_id, level)?;
    let mut archive = archive.clone();
    apply_capture_preview_with_retention(&mut archive, preview, None, policy)
}

pub(in crate::scene::dynamic_scene::session) fn preview_level_slot_with_tag_retention(
    archive: &RuntimeSessionArchive,
    tag: &str,
    slot_id: impl Into<String>,
    level: &LevelSystem,
    policy: RuntimeSessionArchiveRetentionPolicy,
) -> Result<RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError> {
    let preview = slot_capture::preview_level_slot(archive, slot_id, level)?;
    let mut archive = archive.clone();
    apply_capture_preview_with_retention(&mut archive, preview, Some(tag), policy)
}
