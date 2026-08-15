use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::ui::retained_host::primitives::{Color, PhysicalSize, SharedString};
use zircon_runtime_interface::resource::{ResourceKind, ResourceState};

use crate::core::project::RecentProjectValidation;
use crate::ui::animation_editor::AnimationEditorPanePresentation;
use crate::ui::asset_editor::UiAssetEditorPanePresentation;
use crate::ui::layouts::windows::workbench_host_window::{
    BuildExportPaneViewData, ModulePluginsPaneViewData,
};
use crate::ui::retained_host::callback_dispatch::{
    load_startup_builtin_template_runtime, BuiltinHostWindowTemplateBridge,
    BuiltinWorkbenchWindowLayoutFrames, BuiltinWorkbenchWindowTemplateSurfaceBridge,
};
use crate::ui::retained_host::floating_window_projection::build_floating_window_projection_bundle;
use crate::ui::retained_host::{
    apply_presentation, paint_componentized_extension_workspace_for_test,
    paint_host_frame_for_test, paint_template_nodes_for_test_with_background, FrameRect,
    HostChromeControlFrameData, HostChromeTabData, HostClosePromptData, HostMenuChromeData,
    HostMenuChromeItemData, HostMenuChromeMenuData, HostMenuStateData,
    HostPageOverflowMenuStateData, HostWindowLayoutData, TabData, TemplateNodeFrameData,
    TemplatePaneNodeData, UiHostContext, UiHostWindow,
};
use crate::ui::workbench::autolayout::{
    compute_workbench_shell_geometry, ShellSizePx, WorkbenchChromeMetrics,
};
use crate::ui::workbench::fixture::{default_preview_fixture, PreviewFixture};
use crate::ui::workbench::layout::{
    ActivityDrawerMode, ActivityDrawerSlot, MainHostPageLayout, MainPageId, WorkbenchLayout,
};
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::snapshot::{
    AssetFolderSnapshot, AssetItemSnapshot, AssetReferenceSnapshot, AssetSelectionSnapshot,
    AssetSubassetSnapshot, AssetTypeProjectionSnapshot, AssetUtilityTab, AssetViewMode,
    AssetWorkspaceSnapshot, EditorChromeSnapshot,
};
use crate::ui::workbench::startup::{
    EditorSessionMode, NewProjectFormSnapshot, RecentProjectItemSnapshot, WelcomePaneSnapshot,
    WELCOME_DESCRIPTOR_ID, WELCOME_INSTANCE_ID, WELCOME_PAGE_ID,
};
use crate::ui::workbench::view::{
    ViewDescriptor, ViewDescriptorId, ViewHost, ViewInstance, ViewInstanceId, ViewKind,
    WorkbenchSlot,
};
use zircon_runtime_interface::ui::layout::UiSize;

mod asset_browser_content;
mod assets_drawer;
mod blend_space_workspace;
mod chrome_artifacts;
mod component_atlas;
mod fixture_support;
mod layout_assertions;
mod window_fixtures;

use assets_drawer::assets_drawer_window;
use fixture_support::*;
use layout_assertions::assert_assets_drawer_adaptive_layout;
use window_fixtures::*;

