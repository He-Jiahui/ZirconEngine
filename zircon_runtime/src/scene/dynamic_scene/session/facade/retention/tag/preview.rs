use super::super::super::super::retention;
use super::super::super::super::*;

impl RuntimeSessionArchive {
    /// 返回绑定当前代际和修订号的标签裁剪计划；后续目标有变更时提交会拒绝。
    pub fn prepare_prune_slots_with_tag(
        &self,
        tag: &str,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchivePrunePlan, RuntimeSessionArchiveError> {
        retention::prepare_prune_slots_with_tag(self, tag, policy)
    }

    /// 返回只读标签裁剪报告；报告本身不绑定后续提交，桶外槽位也列入保留集合。
    pub fn preview_prune_slots_with_tag(
        &self,
        tag: &str,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
        retention::preview_prune_slots_with_tag(self, tag, policy)
    }
}
