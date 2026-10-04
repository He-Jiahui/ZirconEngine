use std::path::Path;

use super::super::super::super::{
    path_merge, RuntimeSessionArchive, RuntimeSessionArchiveError,
    RuntimeSessionArchiveMergePolicy, RuntimeSessionArchiveMergeReport,
};

impl RuntimeSessionArchive {
    /// 先拒绝源与目标路径相等或已有路径的规范化结果相等，再读来源并合入已有目标；只发布目标文件。
    /// 路径预检不锁定来源，后续文件系统变化仍由读取和保存阶段处理。
    pub fn merge_archive_from_path_at_path_atomically(
        path: impl AsRef<Path>,
        source_path: impl AsRef<Path>,
        policy: RuntimeSessionArchiveMergePolicy,
    ) -> Result<RuntimeSessionArchiveMergeReport, RuntimeSessionArchiveError> {
        path_merge::merge_archive_from_path_at_path_atomically(path, source_path, policy)
    }
}
