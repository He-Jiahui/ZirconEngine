/// 同 ID 槽位的合并决策；默认拒绝冲突，使导入方显式选择保留或替换现有快照。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeSessionArchiveMergePolicy {
    RejectConflicts,
    KeepExisting,
    ReplaceExisting,
}

impl Default for RuntimeSessionArchiveMergePolicy {
    fn default() -> Self {
        Self::RejectConflicts
    }
}
