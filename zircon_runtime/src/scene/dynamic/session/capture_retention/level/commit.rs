use crate::scene::LevelSystem;

use super::super::super::{
    slot_capture, RuntimeSessionArchive, RuntimeSessionArchiveCaptureRetentionReport,
    RuntimeSessionArchiveError, RuntimeSessionArchiveRetentionPolicy,
};
use super::super::apply::prepare_capture_preview_with_retention;

pub(in crate::scene::dynamic_scene::session) fn capture_level_slot_with_retention(
    archive: &mut RuntimeSessionArchive,
    slot_id: impl Into<String>,
    level: &LevelSystem,
    policy: RuntimeSessionArchiveRetentionPolicy,
) -> Result<RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError> {
    let preview = slot_capture::preview_level_slot(archive, slot_id, level)?;
    prepare_capture_preview_with_retention(archive, preview, None, policy)?.commit(archive)
}

pub(in crate::scene::dynamic_scene::session) fn capture_level_slot_with_tag_retention(
    archive: &mut RuntimeSessionArchive,
    tag: &str,
    slot_id: impl Into<String>,
    level: &LevelSystem,
    policy: RuntimeSessionArchiveRetentionPolicy,
) -> Result<RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError> {
    let preview = slot_capture::preview_level_slot(archive, slot_id, level)?;
    prepare_capture_preview_with_retention(archive, preview, Some(tag), policy)?.commit(archive)
}
