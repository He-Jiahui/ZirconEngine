use std::path::Path;

use super::super::super::{
    io, RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchivePruneReport,
    RuntimeSessionArchiveRetentionPolicy, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    pub fn prune_slots_with_tag_at_path_atomically(
        path: impl AsRef<Path>,
        tag: &str,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
        io::mutate_archive_at_path_with_report_atomically(path, |archive| {
            archive.prune_slots_with_tag(tag, policy)
        })
    }

    /// 先规划普通标签裁剪，再额外保留选中的桶内槽位，因此可能超过普通保留数量。
    /// 桶外选择不改变裁剪集合，选择失败则不保存；只有最终档案会发布到目标路径。
    pub fn prune_slots_with_tag_and_selected_protection_at_path_atomically(
        path: impl AsRef<Path>,
        tag: &str,
        policy: RuntimeSessionArchiveRetentionPolicy,
        selector: RuntimeSessionSlotSelector,
    ) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
        io::mutate_archive_at_path_with_report_atomically(path, |archive| {
            archive.prune_slots_with_tag_and_selected_protection(tag, policy, selector)
        })
    }
}
