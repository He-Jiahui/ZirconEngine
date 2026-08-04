use super::super::super::super::merge;
use super::super::super::super::*;

impl RuntimeSessionArchive {
    pub fn commit_merge_plan(
        &mut self,
        plan: RuntimeSessionArchiveMergePlan<'_>,
    ) -> Result<RuntimeSessionArchiveMergeReport, RuntimeSessionArchiveError> {
        plan.commit(self)
    }

    pub fn merge_archive(
        &mut self,
        incoming: &RuntimeSessionArchive,
        policy: RuntimeSessionArchiveMergePolicy,
    ) -> Result<RuntimeSessionArchiveMergeReport, RuntimeSessionArchiveError> {
        merge::merge_archive(self, incoming, policy)
    }
}
