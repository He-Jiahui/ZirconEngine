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
