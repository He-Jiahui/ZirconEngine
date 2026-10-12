use crate::ui::retained_host::primitives::SharedString;

use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload};
use crate::ui::workbench::autolayout::WorkbenchShellGeometry;
use crate::ui::workbench::layout::ActivityDrawerSlot;
use crate::ui::workbench::model::{PaneActionModel, PaneEmptyStateModel, WorkbenchViewModel};
use crate::ui::workbench::snapshot::{
    EditorChromeSnapshot, MainPageSnapshot, ViewContentKind, ViewTabSnapshot,
};

#[path = "projection.rs"]
mod chrome_template_projection;
#[path = "document_leaves.rs"]
mod document_leaves;
#[path = "floating_windows.rs"]
mod floating_windows;
#[path = "frame_rect.rs"]
mod frame_rect;
#[path = "host_data.rs"]
mod host_data;
#[path = "pane_payload.rs"]
mod pane_payload;
#[path = "payloads/mod.rs"]
mod pane_payload_builders;
#[path = "pane_presentation.rs"]
mod pane_presentation;
#[path = "pane_projection.rs"]
mod pane_projection;
#[path = "projection_cache.rs"]
mod projection_cache;
#[path = "scene_projection.rs"]
mod scene_projection;
#[path = "shell_content_selection.rs"]
mod shell_content_selection;
#[path = "shell_presentation.rs"]
mod shell_presentation;

pub(crate) use floating_windows::{
    collect_floating_windows, collect_floating_windows_with_template_v2_data,
};
pub(crate) use frame_rect::frame_rect;
pub(crate) use host_data::{
    AnimationEditorPaneViewData, AssetBrowserPaneViewData, AssetsActivityPaneViewData,
    BuildExportPaneViewData, BuildExportTargetViewData, ConsolePaneViewData, FloatingWindowData,
    FrameRect, GeneratedBottomPaneViewData, HierarchyPaneViewData, HostBottomDockSurfaceData,
    HostChromeControlFrameData, HostChromeTabData, HostDocumentDockSurfaceData,
    HostFloatingWindowLayerData, HostMenuChromeData, HostMenuChromeItemData,
    HostMenuChromeMenuData, HostNativeFloatingWindowSurfaceData, HostPageChromeData,
    HostResizeLayerData, HostSideDockSurfaceData, HostStatusBarData, HostTabDragOverlayData,
    HostWindowLayoutData, HostWindowSceneData, HostWindowShellData, HostWindowSurfaceData,
    HostWindowSurfaceMetricsData, HostWindowSurfaceOrchestrationData, InspectorPaneViewData,
    InspectorPluginComponentPropertyViewData, InspectorPluginComponentViewData,
    ModulePluginStatusViewData, ModulePluginsPaneViewData, PaneContentSize, PaneData,
    PaneNativeBodyData, PerformanceTimelineCaptureControlViewData,
    PerformanceTimelineFrameRowViewData, PerformanceTimelineHotspotRowViewData,
    PerformanceTimelinePaneViewData, PerformanceTimelineSpanRowViewData, ProjectOverviewData,
    ProjectOverviewPaneViewData, SceneNodeData, TabData,
};
#[cfg(test)]
pub(crate) use pane_payload::PerformanceTimelineCaptureControlPayload;
pub(crate) use pane_payload::{PanePayload, RuntimeDiagnosticsPanePayload};
pub(crate) use pane_presentation::{
    build_pane_body_presentation, PaneActionPresentation, PaneBodyPresentation,
    PaneEmptyStatePresentation, PanePayloadBuildContext, PanePresentation, PaneShellPresentation,
};
pub(crate) use pane_projection::{
    blank_pane, document_pane, document_pane_with_template_v2_data, find_tab_snapshot,
    side_pane_with_template_v2_data,
};
pub(crate) use projection_cache::HostChromeProjectionCache;
pub(crate) use scene_projection::{
    build_host_dock_surface_patch, build_host_scene_data_with_cache, HostDockSurfaceId,
    HostDockSurfacePatch,
};
pub(crate) use scene_projection::{
    build_host_scene_data, build_host_scene_geometry, build_native_floating_surface_data,
};
pub(crate) use shell_content_selection::{document_pane_selection, side_pane_selection};
pub(crate) use shell_presentation::{build_host_window_shell_data, ShellPresentation};
