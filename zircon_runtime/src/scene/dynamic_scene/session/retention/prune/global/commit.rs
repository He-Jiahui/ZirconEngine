use super::super::super::super::{RuntimeSessionArchive, RuntimeSessionArchiveError};
use super::super::super::{RuntimeSessionArchivePruneReport, RuntimeSessionArchiveRetentionPolicy};
use super::super::planning::{apply_prune_report, preview_matching_slots};

pub(in crate::scene::dynamic_scene::session) fn prune_slots(
    archive: &mut RuntimeSessionArchive,
    policy: RuntimeSessionArchiveRetentionPolicy,
) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
    archive.ensure_supported()?;
    archive.sort_slots();
    let policy = policy.normalized();
    let report = preview_matching_slots(archive, policy, |_| true)?;
    apply_prune_report(archive, &report);
    Ok(report)
}
