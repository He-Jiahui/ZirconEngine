use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

use slint::{Image, ModelRc, SharedString, VecModel};
use zircon_editor_ui::{EditorUiBinding, EditorUiBindingPayload};
use zircon_manager::ResourceStateRecord;
use zircon_resource::ResourceKind;

use crate::layout::ActivityDrawerSlot;
use crate::snapshot::{
    AssetUtilityTab, AssetViewMode, AssetWorkspaceSnapshot, EditorChromeSnapshot, MainPageSnapshot,
    ProjectOverviewSnapshot, ViewContentKind, ViewTabSnapshot,
};
use crate::workbench::model::{
    DocumentTabModel, HostPageTabModel, PaneActionModel, PaneEmptyStateModel, WorkbenchViewModel,
};
use crate::workbench::startup::RecentProjectValidation;
use crate::{ShellFrame, ShellRegionId, WorkbenchShellGeometry};

use super::tab_drag::{floating_window_edge_group_key, floating_window_group_key};
use super::{
    AssetFolderData, AssetItemData, AssetReferenceData, AssetSelectionData, BreadcrumbData,
    FloatingWindowData, FrameRect, NewProjectFormData, PaneData, ProjectOverviewData,
    RecentProjectData, SceneNodeData, SceneViewportChromeData, TabData, UiAssetCanvasNodeData,
    UiAssetCanvasSlotTargetData, WelcomePaneData, WorkbenchShell,
};

mod apply_presentation;
mod asset_surface_presentation;
mod floating_windows;
mod model_rc;
mod pane_projection;
mod project_overview;
mod shell_presentation;
#[cfg(test)]
mod tests;
mod viewport_chrome;
mod welcome_presentation;
mod workbench_tabs;

pub(crate) use apply_presentation::apply_presentation;
