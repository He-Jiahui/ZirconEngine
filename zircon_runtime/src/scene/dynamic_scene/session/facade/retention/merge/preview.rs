use super::super::super::super::merge;
use super::super::super::super::*;

impl RuntimeSessionArchive {
    pub fn prepare_merge_archive<'incoming>(
        &self,
        incoming: &'incoming RuntimeSessionArchive,
        policy: RuntimeSessionArchiveMergePolicy,
    ) -> Result<RuntimeSessionArchiveMergePlan<'incoming>, RuntimeSessionArchiveError> {
        merge::prepare_merge_archive(self, incoming, policy)
    }

    pub fn preview_merge_archive(
        &self,
        incoming: &RuntimeSessionArchive,
        policy: RuntimeSessionArchiveMergePolicy,
    ) -> Result<RuntimeSessionArchiveMergeReport, RuntimeSessionArchiveError> {
        merge::preview_merge_archive(self, incoming, policy)
    }
}
