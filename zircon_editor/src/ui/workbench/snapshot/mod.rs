//! UI-facing snapshots for editor data and workbench layout binding.

mod asset;
mod data;
mod workbench;

#[allow(unused_imports)]
pub use super::startup::{NewProjectFormSnapshot, RecentProjectItemSnapshot, WelcomePaneSnapshot};
pub use asset::{
    AssetFolderSnapshot, AssetItemSnapshot, AssetReferenceSnapshot, AssetSelectionSnapshot,
    AssetSurfaceMode, AssetUtilityTab, AssetViewMode, AssetWorkspaceSnapshot,
};
pub use data::{
    EditorChromeSnapshot, EditorDataSnapshot, InspectorPluginComponentPropertySnapshot,
    InspectorPluginComponentSnapshot, InspectorSnapshot, ProjectOverviewSnapshot, SceneEntry,
};
pub use workbench::{
    ActivityDrawerSnapshot, DocumentWorkspaceSnapshot, FloatingWindowSnapshot, MainPageSnapshot,
    ViewContentKind, ViewTabSnapshot, WorkbenchSnapshot,
};
