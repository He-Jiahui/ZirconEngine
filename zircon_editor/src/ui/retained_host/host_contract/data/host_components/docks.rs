use crate::ui::retained_host::primitives::{ModelRc, SharedString};

use super::super::{PaneData, TemplatePaneNodeData};
use super::common::{FrameRect, HostChromeControlFrameData, HostChromeTabData, TabData};

#[derive(Clone, Default)]
pub(crate) struct HostResizeLayerData {
    pub left_splitter_frame: FrameRect,
    pub right_splitter_frame: FrameRect,
    pub bottom_splitter_frame: FrameRect,
}

#[derive(Clone, Default)]
pub(crate) struct HostSideDockSurfaceData {
    pub region_frame: FrameRect,
    pub surface_key: SharedString,
    pub rail_before_panel: bool,
    pub rail_nodes: ModelRc<TemplatePaneNodeData>,
    pub rail_button_frames: ModelRc<HostChromeControlFrameData>,
    pub rail_active_control_id: SharedString,
    pub header_nodes: ModelRc<TemplatePaneNodeData>,
    pub header_frame: FrameRect,
    pub content_frame: FrameRect,
    pub tab_frames: ModelRc<HostChromeTabData>,
    pub tabs: ModelRc<TabData>,
    pub pane: PaneData,
    pub rail_width_px: f32,
    pub panel_width_px: f32,
    pub panel_header_height_px: f32,
}

#[derive(Clone, Default)]
pub(crate) struct HostDocumentDockSurfaceData {
    pub region_frame: FrameRect,
    pub surface_key: SharedString,
    pub header_nodes: ModelRc<TemplatePaneNodeData>,
    pub header_frame: FrameRect,
    pub subtitle_frame: FrameRect,
    pub content_frame: FrameRect,
    pub tab_frames: ModelRc<HostChromeTabData>,
    pub tabs: ModelRc<TabData>,
    pub pane: PaneData,
    pub header_height_px: f32,
}

#[derive(Clone, Default)]
pub(crate) struct HostBottomDockSurfaceData {
    pub region_frame: FrameRect,
    pub surface_key: SharedString,
    pub header_nodes: ModelRc<TemplatePaneNodeData>,
    pub header_frame: FrameRect,
    pub content_frame: FrameRect,
    pub tab_frames: ModelRc<HostChromeTabData>,
    pub tabs: ModelRc<TabData>,
    pub pane: PaneData,
    pub expanded: bool,
    pub header_height_px: f32,
}
