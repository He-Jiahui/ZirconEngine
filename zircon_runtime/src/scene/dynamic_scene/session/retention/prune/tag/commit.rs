use super::super::super::super::{RuntimeSessionArchive, RuntimeSessionArchiveError};
use super::super::super::{RuntimeSessionArchivePruneReport, RuntimeSessionArchiveRetentionPolicy};
use super::super::planning::{apply_prune_report, preview_matching_slots_with_tag};

pub(in crate::scene::dynamic_scene::session) fn prune_slots_with_tag(
    archive: &mut RuntimeSessionArchive,
    tag: &str,
    policy: RuntimeSessionArchiveRetentionPolicy,
) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
    archive.ensure_supported()?;
    archive.sort_slots();
    let tag = tag.trim();
    let policy = policy.normalized();
    if tag.is_empty() {
        return Ok(RuntimeSessionArchivePruneReport {
            retained_slot_ids: archive.slot_ids().map(str::to_string).collect(),
            removed_slot_ids: Vec::new(),
        });
    }
    let report = preview_matching_slots_with_tag(archive, tag, policy)?;
    apply_prune_report(archive, &report);
    Ok(report)
}
