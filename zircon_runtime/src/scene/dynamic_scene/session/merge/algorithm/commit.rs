use super::super::super::{RuntimeSessionArchive, RuntimeSessionArchiveError};
use super::super::{RuntimeSessionArchiveMergePolicy, RuntimeSessionArchiveMergeReport};
use super::plan::prepare_merge_archive;

pub(in crate::scene::dynamic_scene::session) fn merge_archive(
    target: &mut RuntimeSessionArchive,
    incoming: &RuntimeSessionArchive,
    policy: RuntimeSessionArchiveMergePolicy,
) -> Result<RuntimeSessionArchiveMergeReport, RuntimeSessionArchiveError> {
    let prepared = prepare_merge_archive(target, incoming, policy)?;
    prepared.commit(target)
}
