use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetWatchError {
    pub assets_root: PathBuf,
    pub paths: Vec<PathBuf>,
    pub message: String,
}

impl AssetWatchError {
    pub(super) fn from_notify_error(assets_root: PathBuf, error: notify::Error) -> Self {
        let message = error.to_string();
        Self {
            assets_root,
            paths: error.paths,
            message,
        }
    }

    pub(crate) fn from_message(assets_root: PathBuf, message: impl Into<String>) -> Self {
        Self {
            assets_root,
            paths: Vec::new(),
            message: message.into(),
        }
    }
}

#[cfg(test)]
#[path = "tests/asset_watch_error_optimization_batch_gt_runtime575_tests.rs"]
mod optimization_batch_gt_runtime575_tests;
