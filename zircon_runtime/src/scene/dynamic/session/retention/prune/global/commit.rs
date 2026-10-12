use super::super::super::super::{RuntimeSessionArchive, RuntimeSessionArchiveError};
use super::super::super::{RuntimeSessionArchivePruneReport, RuntimeSessionArchiveRetentionPolicy};
use super::super::planning::prepare_prune_slots;

pub(in crate::scene::dynamic_scene::session) fn prune_slots(
    archive: &mut RuntimeSessionArchive,
    policy: RuntimeSessionArchiveRetentionPolicy,
) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
    prepare_prune_slots(archive, policy)?.commit(archive)
}
