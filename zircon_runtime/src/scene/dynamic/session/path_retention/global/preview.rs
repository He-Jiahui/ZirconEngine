use std::path::Path;

use super::super::super::{
    io, RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchivePruneReport,
    RuntimeSessionArchiveRetentionPolicy, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    pub fn preview_prune_slots_from_path(
        path: impl AsRef<Path>,
        policy: RuntimeSessionArchiveRetentionPolicy,
    ) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
        io::load_from_path(path)?.preview_prune_slots(policy)
    }

    /// 在当前读取的档案上解析选择器并预览全局保护；不写文件或绑定后续选择结果。
    pub fn preview_prune_slots_with_selected_protection_from_path(
        path: impl AsRef<Path>,
        policy: RuntimeSessionArchiveRetentionPolicy,
        selector: RuntimeSessionSlotSelector,
    ) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
        io::load_from_path(path)?.preview_prune_slots_with_selected_protection(policy, selector)
    }
}
