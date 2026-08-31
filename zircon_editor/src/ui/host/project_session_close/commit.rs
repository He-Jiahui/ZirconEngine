use std::path::{Path, PathBuf};

use super::ProjectCloseReceipt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProjectCloseCommit {
    closed_root: Option<PathBuf>,
    receipt: ProjectCloseReceipt,
}

impl ProjectCloseCommit {
    pub(crate) fn new(closed_root: Option<PathBuf>, receipt: ProjectCloseReceipt) -> Self {
        Self {
            closed_root,
            receipt,
        }
    }

    pub(crate) fn closed_root(&self) -> Option<&Path> {
        self.closed_root.as_deref()
    }

    pub(crate) fn into_closed_root(self) -> Option<PathBuf> {
        self.closed_root
    }

    pub(crate) fn receipt(&self) -> &ProjectCloseReceipt {
        &self.receipt
    }
}
