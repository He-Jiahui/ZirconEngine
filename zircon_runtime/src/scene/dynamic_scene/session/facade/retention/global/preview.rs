use super::super::super::super::retention;
use super::super::super::super::*;

impl RuntimeSessionArchive {
    /// 生成与当前代际绑定的裁剪计划；预览后若档案变化，提交必须报告过期而非删错槽位。
    pub fn prepare_prune_slots(
        &self,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchivePrunePlan, RuntimeSessionArchiveError> {
        retention::prepare_prune_slots(self, policy)
    }

    pub fn preview_prune_slots(
        &self,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
        retention::preview_prune_slots(self, policy)
    }
}
