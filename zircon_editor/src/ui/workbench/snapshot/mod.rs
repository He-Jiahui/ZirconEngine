//! UI-facing snapshots for editor data and workbench layout binding.

#[path = "asset/mod.rs"]
mod asset;
#[path = "data/mod.rs"]
mod data;
#[path = "workbench/mod.rs"]
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
    EditorConsoleMessageLevel, EditorDataSnapshot, InspectorNativeFieldSnapshot,
    InspectorPluginComponentPropertySnapshot, InspectorPluginComponentSnapshot, InspectorSnapshot,
    ProjectOverviewSnapshot, SceneEntries, SceneEntry, StatusTaskProgressSnapshot,
    StatusSignalsSnapshot, StatusTaskProgressTone, TransactionHistoryRowSnapshot, TransactionHistorySnapshot,
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
