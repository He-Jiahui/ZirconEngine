use std::path::Path;

use super::super::super::super::{
    path_merge, RuntimeSessionArchive, RuntimeSessionArchiveError,
    RuntimeSessionArchiveMergePolicy, RuntimeSessionArchiveMergeReport,
};

impl RuntimeSessionArchive {
    /// 读取已有目标档案并合并借用的来源，保存完成后才返回报告；来源不被修改。
    /// 即使内存合并没有插入或替换，也会保存目标；原子发布不锁定整个读改写过程。
    pub fn merge_archive_at_path_atomically(
        path: impl AsRef<Path>,
        incoming: &RuntimeSessionArchive,
        policy: RuntimeSessionArchiveMergePolicy,
    ) -> Result<RuntimeSessionArchiveMergeReport, RuntimeSessionArchiveError> {
        path_merge::merge_archive_at_path_atomically(path, incoming, policy)
    }
}
