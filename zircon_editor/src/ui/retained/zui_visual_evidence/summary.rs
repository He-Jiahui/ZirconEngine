use std::path::{Path, PathBuf};

/// Native rendering readiness; acceptance is recorded by the dual renderer review.
#[derive(Debug)]
pub struct ZuiVisualEvidenceSummary {
    pub(super) captured: usize,
    pub(super) failed: usize,
    pub(super) pending: usize,
    pub(super) report_path: PathBuf,
}

impl ZuiVisualEvidenceSummary {
    pub fn captured(&self) -> usize {
        self.captured
    }

    pub fn failed(&self) -> usize {
        self.failed
    }

    pub fn pending(&self) -> usize {
        self.pending
    }

    pub fn is_ready(&self) -> bool {
        self.captured > 0 && self.failed == 0 && self.pending == 0
    }

    /// Capturing native pixels cannot certify Penpot parity or visual review.
    pub fn is_accepted(&self) -> bool {
        false
    }

    pub fn report_path(&self) -> &Path {
        &self.report_path
    }
}

#[cfg(test)]
#[path = "summary/tests/cases.rs"]
mod tests;
