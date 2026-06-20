use crate::ui::retained_host::primitives::{ModelRc, SharedString};

use super::super::PaneData;
use super::common::{FrameRect, TabData};
use super::floating::FloatingWindowData;

#[derive(Clone, Default)]
pub(crate) struct HostWindowShellData {
    pub project_path: SharedString,
    pub status_secondary: SharedString,
    pub debug_refresh_rate: SharedString,
    pub viewport_label: SharedString,
    pub drawers_visible: bool,
    pub left_expanded: bool,
    pub right_expanded: bool,
    pub bottom_expanded: bool,
    pub save_project_enabled: bool,
    pub undo_enabled: bool,
    pub redo_enabled: bool,
    pub preset_names: ModelRc<SharedString>,
    pub active_preset_name: SharedString,
    pub skin_id: SharedString,
    pub panel_preset_id: SharedString,
    pub shell_preset_id: SharedString,
    pub window_model_preset_id: SharedString,
    pub shell_min_width_px: f32,
    pub shell_min_height_px: f32,
    pub native_floating_window_mode: bool,
    pub native_floating_window_id: SharedString,
    pub native_surface_tree_id: SharedString,
    pub native_window_title: SharedString,
    pub native_window_bounds: FrameRect,
}

#[derive(Clone, Default)]
pub(crate) struct HostWindowLayoutData {
    pub center_band_frame: FrameRect,
    pub status_bar_frame: FrameRect,
    pub left_region_frame: FrameRect,
    pub document_region_frame: FrameRect,
    pub right_region_frame: FrameRect,
    pub bottom_region_frame: FrameRect,
    pub left_splitter_frame: FrameRect,
    pub right_splitter_frame: FrameRect,
    pub bottom_splitter_frame: FrameRect,
    pub viewport_content_frame: FrameRect,
}

#[derive(Clone, Default)]
pub(crate) struct HostWindowBootstrapData {
    pub shell_frame: FrameRect,
    pub viewport_content_frame: FrameRect,
}

#[derive(Clone, Default)]
pub(crate) struct HostWindowSurfaceData {
    pub host_tabs: ModelRc<TabData>,
    pub left_tabs: ModelRc<TabData>,
    pub right_tabs: ModelRc<TabData>,
    pub bottom_tabs: ModelRc<TabData>,
    pub document_tabs: ModelRc<TabData>,
    pub floating_windows: ModelRc<FloatingWindowData>,
    pub left_pane: PaneData,
    pub right_pane: PaneData,
    pub bottom_pane: PaneData,
    pub document_pane: PaneData,
}

#[derive(Clone, Default)]
pub(crate) struct HostTabDragOverlayData {
    pub left_drop_enabled: bool,
    pub right_drop_enabled: bool,
    pub bottom_drop_enabled: bool,
    pub left_drop_width_px: f32,
    pub right_drop_width_px: f32,
    pub bottom_drop_height_px: f32,
    pub main_content_y_px: f32,
    pub main_content_height_px: f32,
    pub document_zone_x_px: f32,
    pub document_zone_width_px: f32,
    pub bottom_drop_top_px: f32,
    pub drag_overlay_bottom_px: f32,
}
