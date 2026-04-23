use slint::SharedString;

use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload};
use crate::ui::workbench::autolayout::WorkbenchShellGeometry;
use crate::ui::workbench::layout::ActivityDrawerSlot;
use crate::ui::workbench::model::{PaneActionModel, PaneEmptyStateModel, WorkbenchViewModel};
use crate::ui::workbench::snapshot::{
    EditorChromeSnapshot, MainPageSnapshot, ViewContentKind, ViewTabSnapshot,
};

mod floating_windows;
mod frame_rect;
mod host_data;
mod pane_projection;
mod scene_projection;
mod shell_presentation;

pub(crate) use floating_windows::collect_floating_windows;
pub(crate) use frame_rect::frame_rect;
pub(crate) use host_data::{
    AnimationEditorPaneViewData, AssetBrowserPaneViewData, AssetsActivityPaneViewData,
    ConsolePaneViewData, FloatingWindowData, FrameRect, HierarchyPaneViewData,
    HostBottomDockSurfaceData, HostDocumentDockSurfaceData, HostFloatingWindowLayerData,
    HostMenuChromeData, HostNativeFloatingWindowSurfaceData, HostPageChromeData,
    HostResizeLayerData, HostSideDockSurfaceData, HostStatusBarData, HostTabDragOverlayData,
    HostWindowLayoutData, HostWindowShellData, HostWindowSurfaceData,
    HostWorkbenchSurfaceMetricsData, HostWorkbenchSurfaceOrchestrationData,
    HostWorkbenchWindowSceneData, InspectorPaneViewData, PaneData, ProjectOverviewData,
    ProjectOverviewPaneViewData, SceneNodeData, TabData,
};
pub(crate) use pane_projection::document_pane;
pub(crate) use scene_projection::{build_host_scene_data, build_native_floating_surface_data};
pub(crate) use shell_presentation::ShellPresentation;
