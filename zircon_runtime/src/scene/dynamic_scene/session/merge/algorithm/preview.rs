use super::super::super::{RuntimeSessionArchive, RuntimeSessionArchiveError};
use super::super::{RuntimeSessionArchiveMergePolicy, RuntimeSessionArchiveMergeReport};
use super::plan::prepare_merge_archive;

pub(in crate::scene::dynamic_scene::session) fn preview_merge_archive(
    target: &RuntimeSessionArchive,
    incoming: &RuntimeSessionArchive,
    policy: RuntimeSessionArchiveMergePolicy,
) -> Result<RuntimeSessionArchiveMergeReport, RuntimeSessionArchiveError> {
    Ok(prepare_merge_archive(target, incoming, policy)?
        .report()
        .clone())
}
