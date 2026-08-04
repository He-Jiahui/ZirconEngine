use super::super::super::super::{RuntimeSessionArchive, RuntimeSessionArchiveError};
use super::super::super::{RuntimeSessionArchivePruneReport, RuntimeSessionArchiveRetentionPolicy};
use super::super::planning::preview_matching_slots;

pub(in crate::scene::dynamic_scene::session) fn prune_slots(
    archive: &mut RuntimeSessionArchive,
    policy: RuntimeSessionArchiveRetentionPolicy,
) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
    archive.ensure_supported()?;
    let policy = policy.normalized();
    let report = preview_matching_slots(archive, policy, |_| true)?;
    if !report.removed_slot_ids.is_empty() {
        archive.commit_staged_slot_rows(
            Vec::new(),
            Vec::new(),
            report.removed_slot_ids.iter().map(String::as_str),
        );
    }
    Ok(report)
}
