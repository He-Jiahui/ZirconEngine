use std::path::PathBuf;

use super::super::{
    RuntimeSessionArchiveManifest, RuntimeSessionArchivePruneReport,
    RuntimeSessionArchiveStatistics,
};
use super::capture::RuntimeSessionSlotCapturePreviewReport;

/// 保存前的目标状态与快照统计；目标是否被替换只是预览时的观察，提交时须重新核查。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeSessionArchiveSavePreviewReport {
    pub target_path: PathBuf,
    pub will_replace_target: bool,
    pub statistics: RuntimeSessionArchiveStatistics,
}

/// 捕获与裁剪的联合报告；manifest 描述预览或提交后的逻辑档案，不是另一次文件读取。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeSessionArchiveCaptureRetentionReport {
    pub capture: RuntimeSessionSlotCapturePreviewReport,
    pub prune: RuntimeSessionArchivePruneReport,
    pub manifest: RuntimeSessionArchiveManifest,
}
