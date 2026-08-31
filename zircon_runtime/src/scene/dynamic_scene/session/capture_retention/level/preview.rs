use crate::scene::LevelSystem;

use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError,
    RuntimeSessionArchiveRetentionPolicy, slot_capture,
};
use super::super::apply::prepare_capture_preview_with_retention;

pub(in crate::scene::dynamic_scene::session) fn preview_level_slot_with_retention(
    archive: &RuntimeSessionArchive,
    slot_id: impl Into<String>,
    level: &LevelSystem,
    policy: RuntimeSessionArchiveRetentionPolicy,
) -> Result<RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError> {
    let preview = slot_capture::preview_level_slot(archive, slot_id, level)?;
    Ok(prepare_capture_preview_with_retention(archive, preview, None, policy)?.report(archive))
}

pub(in crate::scene::dynamic_scene::session) fn preview_level_slot_with_tag_retention(
    archive: &RuntimeSessionArchive,
    tag: &str,
    slot_id: impl Into<String>,
    level: &LevelSystem,
    policy: RuntimeSessionArchiveRetentionPolicy,
) -> Result<RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError> {
    let preview = slot_capture::preview_level_slot(archive, slot_id, level)?;
    Ok(
        prepare_capture_preview_with_retention(archive, preview, Some(tag), policy)?
            .report(archive),
    )
}
