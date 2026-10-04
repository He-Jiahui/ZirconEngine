use super::super::super::super::retention;
use super::super::super::super::*;

impl RuntimeSessionArchive {
    /// 只裁剪去除首尾空白后的标签桶，桶外槽位始终保留；空标签查询不删除任何槽位。
    pub fn prune_slots_with_tag(
        &mut self,
        tag: &str,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
        retention::prune_slots_with_tag(self, tag, policy)
    }
}
