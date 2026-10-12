use super::super::super::super::retention;
use super::super::super::super::*;

impl RuntimeSessionArchive {
    /// 按全档案保留规则裁剪内存槽位；保护项先占用保留名额，超过数量上限时仍保留。
    /// 无删除时不推进修订号；磁盘发布需使用路径入口。
    pub fn prune_slots(
        &mut self,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
        retention::prune_slots(self, policy)
    }
}
