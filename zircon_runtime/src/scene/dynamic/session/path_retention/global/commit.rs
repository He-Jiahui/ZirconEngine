use std::path::Path;

use super::super::super::{
    io, RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchivePruneReport,
    RuntimeSessionArchiveRetentionPolicy, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    pub fn prune_slots_at_path_atomically(
        path: impl AsRef<Path>,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
        io::mutate_archive_at_path_with_report_atomically(path, |archive| {
            archive.prune_slots(policy)
        })
    }

    /// 在本次读取的目标上解析选择器，并把选中 ID 加入保护集后规划全档案裁剪。
    /// 选中槽位占用保留名额；选择失败不保存，成功裁剪后才发布目标文件。
    pub fn prune_slots_with_selected_protection_at_path_atomically(
        path: impl AsRef<Path>,
        policy: RuntimeSessionArchiveRetentionPolicy,
        selector: RuntimeSessionSlotSelector,
    ) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
        io::mutate_archive_at_path_with_report_atomically(path, |archive| {
            archive.prune_slots_with_selected_protection(policy, selector)
        })
    }
}
