use super::super::super::super::{RuntimeSessionArchive, RuntimeSessionArchiveError};
use super::super::super::{RuntimeSessionArchivePruneReport, RuntimeSessionArchiveRetentionPolicy};
use super::super::planning::{preview_matching_slots_with_tag, sorted_slot_ids};

pub(in crate::scene::dynamic_scene::session) fn preview_prune_slots_with_tag(
    archive: &RuntimeSessionArchive,
    tag: &str,
    policy: RuntimeSessionArchiveRetentionPolicy,
) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
    archive.ensure_supported()?;
    let tag = tag.trim();
    if tag.is_empty() {
        return Ok(RuntimeSessionArchivePruneReport {
            retained_slot_ids: sorted_slot_ids(archive),
            removed_slot_ids: Vec::new(),
        });
    }
    preview_matching_slots_with_tag(archive, tag, policy.normalized())
}
