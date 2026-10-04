use std::path::Path;

use super::super::super::super::{
    path_merge, RuntimeSessionArchive, RuntimeSessionArchiveError,
    RuntimeSessionArchiveMergePolicy, RuntimeSessionArchiveMergeReport,
};

impl RuntimeSessionArchive {
    /// 分别读取来源和目标生成只读报告；路径相等或已有路径的规范化结果相等时拒绝。
    /// 两次读取不构成共同快照。
    pub fn preview_merge_archive_from_path_at_path(
        path: impl AsRef<Path>,
        source_path: impl AsRef<Path>,
        policy: RuntimeSessionArchiveMergePolicy,
    ) -> Result<RuntimeSessionArchiveMergeReport, RuntimeSessionArchiveError> {
        path_merge::preview_merge_archive_from_path_at_path(path, source_path, policy)
    }
}
