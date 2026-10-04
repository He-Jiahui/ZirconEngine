use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchivePruneReport,
    RuntimeSessionArchiveRetentionPolicy, RuntimeSessionSlotSelector,
};
use super::super::policy::policy_with_selected_protection;

impl RuntimeSessionArchive {
    /// 在全档案裁剪规划前保护选择器命中的槽位，已有保护项继续有效。
    /// 保护槽位参与保留数量规划；超过上限的保护项仍会保留，选择器未命中则返回错误。
    /// 仅提交当前内存档案；磁盘发布由路径裁剪入口负责。
    pub fn prune_slots_with_selected_protection(
        &mut self,
        policy: RuntimeSessionArchiveRetentionPolicy,
        selector: RuntimeSessionSlotSelector,
    ) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
        let policy = policy_with_selected_protection(self, policy, selector)?;
        self.prune_slots(policy)
    }
}
