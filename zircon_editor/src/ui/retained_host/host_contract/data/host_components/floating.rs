use crate::ui::retained_host::primitives::{ModelRc, SharedString};

use super::super::{PaneData, TemplatePaneNodeData};
use super::common::{FrameRect, HostChromeTabData, TabData};

#[derive(Clone, Default)]
pub(crate) struct FloatingWindowData {
    pub window_id: SharedString,
    pub title: SharedString,
    pub frame: FrameRect,
    pub header_nodes: ModelRc<TemplatePaneNodeData>,
    pub header_frame: FrameRect,
    pub overflow_frame: FrameRect,
    pub tab_frames: ModelRc<HostChromeTabData>,
    pub target_group: SharedString,
    pub left_edge_target_group: SharedString,
    pub right_edge_target_group: SharedString,
    pub top_edge_target_group: SharedString,
    pub bottom_edge_target_group: SharedString,
    pub focus_target_id: SharedString,
    pub tabs: ModelRc<TabData>,
    pub active_pane: PaneData,
}

#[derive(Clone, Default)]
pub(crate) struct HostFloatingWindowLayerData {
    pub floating_windows: ModelRc<FloatingWindowData>,
    pub header_height_px: f32,
}

#[derive(Clone, Default)]
pub(crate) struct HostNativeFloatingWindowSurfaceData {
    pub floating_windows: ModelRc<FloatingWindowData>,
    pub native_floating_window_id: SharedString,
    pub native_surface_tree_id: SharedString,
    pub native_window_bounds: FrameRect,
    pub header_height_px: f32,
}
