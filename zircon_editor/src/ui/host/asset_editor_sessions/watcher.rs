#[path = "watcher/budget.rs"]
mod budget;
#[path = "watcher/diagnostics.rs"]
mod diagnostics;
#[path = "watcher/host.rs"]
mod host;
#[path = "watcher/ingress.rs"]
mod ingress;
#[path = "watcher/path_identity.rs"]
mod path_identity;
#[path = "watcher/service.rs"]
mod service;

#[cfg(test)]
#[path = "watcher/tests/cases.rs"]
mod tests;

pub(super) use budget::UiAssetWatchPollAllowance;
pub use diagnostics::{UiAssetWorkspaceWatchDiagnostics, UiAssetWorkspaceWatchPollReport};
pub(super) use service::UiAssetWatchReconcileCursor;
pub(crate) use service::UiAssetWorkspaceWatcher;
