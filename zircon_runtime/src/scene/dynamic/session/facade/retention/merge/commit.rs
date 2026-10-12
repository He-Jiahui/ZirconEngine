use super::super::super::super::merge;
use super::super::super::super::*;

impl RuntimeSessionArchive {
    /// 提交先前预检的合并决策；目标代际变更时拒绝，避免把旧预览应用到新档案。
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
