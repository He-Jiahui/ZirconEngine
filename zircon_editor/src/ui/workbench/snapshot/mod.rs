//! UI-facing snapshots for editor data and workbench layout binding.

mod asset;
mod data;
mod workbench;

#[allow(unused_imports)]
pub use super::startup::{NewProjectFormSnapshot, RecentProjectItemSnapshot, WelcomePaneSnapshot};
pub use asset::{
    AssetFolderSnapshot, AssetItemSnapshot, AssetOperationProjectionSnapshot,
    AssetReferenceSnapshot, AssetSelectionSnapshot, AssetSubassetSnapshot, AssetSurfaceMode,
    AssetTypeProjectionSnapshot, AssetUtilityTab, AssetViewMode, AssetWorkspaceItemGeneration,
    AssetWorkspaceSnapshot,
};
pub use data::{
    ConsoleOutputLevelCounts, ConsoleOutputSnapshot, EditorBridgeDiagnosticsSnapshot,
    EditorBridgeDiagnosticsSummarySnapshot, EditorBridgeInterfaceRowSnapshot, EditorChromeSnapshot,
    EditorConsoleMessageLevel, EditorDataSnapshot, InspectorPluginComponentPropertySnapshot,
    InspectorPluginComponentSnapshot, InspectorSnapshot, ProjectOverviewSnapshot, SceneEntries,
    SceneEntry, StatusTaskProgressSnapshot, StatusTaskProgressTone, TransactionHistoryRowSnapshot,
    TransactionHistorySnapshot,
};
pub(crate) use data::{
    ConsoleOutputLineDelta, ConsoleOutputLineGeneration, ConsoleOutputLineSnapshot,
    CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY,
};
pub(crate) use data::{
    SceneEntryProjectionCache, SceneInspectionHierarchyFragment,
    SceneInspectionHierarchyFragmentError,
};
pub use workbench::{
    ActivityDrawerSnapshot, DocumentWorkspaceSnapshot, FloatingWindowSnapshot, MainPageSnapshot,
    ViewContentKind, ViewTabSnapshot, WorkbenchSnapshot,
};
