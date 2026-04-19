use slint::{ModelRc, SharedString};

use crate::layout::ActivityDrawerSlot;
use crate::snapshot::{EditorChromeSnapshot, MainPageSnapshot, ViewContentKind, ViewTabSnapshot};
use crate::ui::slint_host::{
    FloatingWindowData, FrameRect, HostWindowShellData, HostWindowSurfaceData, PaneData,
    ProjectOverviewData, SceneNodeData,
};
use crate::ui::workbench::model::{PaneActionModel, PaneEmptyStateModel, WorkbenchViewModel};
use crate::ui::{EditorUiBinding, EditorUiBindingPayload};
use crate::WorkbenchShellGeometry;

mod floating_windows;
mod frame_rect;
mod pane_projection;
mod scene_projection;
mod shell_presentation;

pub(crate) use floating_windows::collect_floating_windows;
pub(crate) use frame_rect::frame_rect;
pub(crate) use pane_projection::document_pane;
pub(crate) use scene_projection::{
    build_host_scene_data, build_native_floating_surface_data,
};
pub(crate) use shell_presentation::ShellPresentation;
