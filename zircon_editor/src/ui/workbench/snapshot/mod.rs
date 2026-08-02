//! UI-facing snapshots for editor data and workbench layout binding.

mod asset;
mod data;
mod workbench;

#[allow(unused_imports)]
pub use super::startup::{NewProjectFormSnapshot, RecentProjectItemSnapshot, WelcomePaneSnapshot};
pub use asset::{
    AssetFolderSnapshot, AssetItemSnapshot, AssetOperationProjectionSnapshot,
    AssetReferenceSnapshot, AssetSelectionSnapshot, AssetSubassetSnapshot, AssetSurfaceMode,
    AssetTypeProjectionSnapshot, AssetUtilityTab, AssetViewMode, AssetWorkspaceSnapshot,
};
pub(crate) use data::CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY;
pub(crate) use data::SceneEntryProjectionCache;
pub use data::{
    ConsoleOutputLevelCounts, ConsoleOutputSnapshot, EditorBridgeDiagnosticsSnapshot,
    EditorBridgeDiagnosticsSummarySnapshot, EditorBridgeInterfaceRowSnapshot, EditorChromeSnapshot,
    EditorConsoleMessageLevel, EditorDataSnapshot, InspectorPluginComponentPropertySnapshot,
    InspectorPluginComponentSnapshot, InspectorSnapshot, ProjectOverviewSnapshot, SceneEntries,
    SceneEntry, StatusTaskProgressSnapshot, StatusTaskProgressTone,
};
pub use workbench::{
    ActivityDrawerSnapshot, DocumentWorkspaceSnapshot, FloatingWindowSnapshot, MainPageSnapshot,
    ViewContentKind, ViewTabSnapshot, WorkbenchSnapshot,
};
