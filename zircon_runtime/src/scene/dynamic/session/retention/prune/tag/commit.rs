use super::super::super::super::{RuntimeSessionArchive, RuntimeSessionArchiveError};
use super::super::super::{RuntimeSessionArchivePruneReport, RuntimeSessionArchiveRetentionPolicy};
use super::super::planning::prepare_prune_slots_with_tag;

pub(in crate::scene::dynamic_scene::session) fn prune_slots_with_tag(
    archive: &mut RuntimeSessionArchive,
    tag: &str,
    policy: RuntimeSessionArchiveRetentionPolicy,
) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
    prepare_prune_slots_with_tag(archive, tag, policy)?.commit(archive)
}
