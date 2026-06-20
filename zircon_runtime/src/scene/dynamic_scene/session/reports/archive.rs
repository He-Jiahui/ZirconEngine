use std::path::PathBuf;

use super::super::{
    RuntimeSessionArchiveManifest, RuntimeSessionArchivePruneReport,
    RuntimeSessionArchiveStatistics,
};
use super::capture::RuntimeSessionSlotCapturePreviewReport;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeSessionArchiveSavePreviewReport {
    pub target_path: PathBuf,
    pub will_replace_target: bool,
    pub statistics: RuntimeSessionArchiveStatistics,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeSessionArchiveCaptureRetentionReport {
    pub capture: RuntimeSessionSlotCapturePreviewReport,
    pub prune: RuntimeSessionArchivePruneReport,
    pub manifest: RuntimeSessionArchiveManifest,
}
