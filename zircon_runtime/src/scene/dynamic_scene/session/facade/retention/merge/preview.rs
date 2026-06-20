use super::super::super::super::merge;
use super::super::super::super::*;

impl RuntimeSessionArchive {
    pub fn preview_merge_archive(
        &self,
        incoming: &RuntimeSessionArchive,
        policy: RuntimeSessionArchiveMergePolicy,
    ) -> Result<RuntimeSessionArchiveMergeReport, RuntimeSessionArchiveError> {
        merge::preview_merge_archive(self, incoming, policy)
    }
}
