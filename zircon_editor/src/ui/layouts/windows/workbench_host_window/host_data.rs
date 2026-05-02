use slint::{ModelRc, SharedString};

use crate::ui::asset_editor::UiAssetEditorPanePresentation;
use crate::ui::layouts::views::{SceneViewportChromeData, ViewTemplateNodeData};

#[derive(Clone, Default)]
pub(crate) struct FrameRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Copy, Default)]
pub(crate) struct PaneContentSize {
    pub width: f32,
    pub height: f32,
}

impl PaneContentSize {
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

#[derive(Clone)]
pub(crate) struct TabData {
    pub id: SharedString,
    pub slot: SharedString,
    pub title: SharedString,
    pub icon_key: SharedString,
    pub active: bool,
    pub closeable: bool,
}

#[derive(Clone)]
pub(crate) struct HostChromeControlFrameData {
    pub control_id: SharedString,
    pub frame: FrameRect,
}

#[derive(Clone)]
pub(crate) struct HostChromeTabData {
    pub control_id: SharedString,
    pub tab: TabData,
    pub frame: FrameRect,
    pub close_frame: FrameRect,
}

#[derive(Clone)]
pub(crate) struct FloatingWindowData {
    pub window_id: SharedString,
    pub title: SharedString,
    pub frame: FrameRect,
    pub header_nodes: ModelRc<ViewTemplateNodeData>,
    pub header_frame: FrameRect,
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

#[derive(Clone)]
pub(crate) struct PaneData {
    pub id: SharedString,
    pub slot: SharedString,
    pub kind: SharedString,
    pub title: SharedString,
    pub icon_key: SharedString,
    pub subtitle: SharedString,
    pub info: SharedString,
    pub show_empty: bool,
    pub empty_title: SharedString,
    pub empty_body: SharedString,
    pub primary_action_label: SharedString,
    pub primary_action_id: SharedString,
    pub secondary_action_label: SharedString,
    pub secondary_action_id: SharedString,
    pub secondary_hint: SharedString,
    pub show_toolbar: bool,
    pub viewport: SceneViewportChromeData,
    pub native_body: PaneNativeBodyData,
    #[allow(dead_code)]
    pub pane_presentation: Option<super::PanePresentation>,
}

#[derive(Clone, Default)]
pub(crate) struct PaneNativeBodyData {
    pub hierarchy: HierarchyPaneViewData,
    pub inspector: InspectorPaneViewData,
    pub console: ConsolePaneViewData,
    pub assets_activity: AssetsActivityPaneViewData,
    pub asset_browser: AssetBrowserPaneViewData,
    pub project_overview: ProjectOverviewPaneViewData,
    pub module_plugins: ModulePluginsPaneViewData,
    pub ui_asset: UiAssetEditorPanePresentation,
    pub animation: AnimationEditorPaneViewData,
}

#[derive(Clone, Default)]
pub(crate) struct HierarchyPaneViewData {
    pub nodes: ModelRc<ViewTemplateNodeData>,
    pub hierarchy_nodes: ModelRc<SceneNodeData>,
}

#[derive(Clone, Default)]
pub(crate) struct InspectorPaneViewData {
    pub nodes: ModelRc<ViewTemplateNodeData>,
    pub info: SharedString,
    pub inspector_name: SharedString,
    pub inspector_parent: SharedString,
    pub inspector_x: SharedString,
    pub inspector_y: SharedString,
    pub inspector_z: SharedString,
    pub delete_enabled: bool,
}

#[derive(Clone, Default)]
pub(crate) struct ConsolePaneViewData {
    pub nodes: ModelRc<ViewTemplateNodeData>,
    pub status_text: SharedString,
}

#[derive(Clone, Default)]
pub(crate) struct AssetsActivityPaneViewData {
    pub nodes: ModelRc<ViewTemplateNodeData>,
}

#[derive(Clone, Default)]
pub(crate) struct AssetBrowserPaneViewData {
    pub nodes: ModelRc<ViewTemplateNodeData>,
}

#[derive(Clone, Default)]
pub(crate) struct ProjectOverviewPaneViewData {
    pub nodes: ModelRc<ViewTemplateNodeData>,
}

#[derive(Clone, Default)]
pub(crate) struct ModulePluginStatusViewData {
    pub plugin_id: SharedString,
    pub display_name: SharedString,
    pub package_source: SharedString,
    pub load_state: SharedString,
    pub enabled: bool,
    pub required: bool,
    pub target_modes: SharedString,
    pub packaging: SharedString,
    pub runtime_crate: SharedString,
    pub editor_crate: SharedString,
    pub runtime_capabilities: SharedString,
    pub editor_capabilities: SharedString,
    pub optional_features: SharedString,
    pub feature_action_label: SharedString,
    pub feature_action_id: SharedString,
    pub diagnostics: SharedString,
    pub primary_action_label: SharedString,
    pub primary_action_id: SharedString,
    pub packaging_action_label: SharedString,
    pub packaging_action_id: SharedString,
    pub target_modes_action_label: SharedString,
    pub target_modes_action_id: SharedString,
    pub unload_action_label: SharedString,
    pub unload_action_id: SharedString,
    pub hot_reload_action_label: SharedString,
    pub hot_reload_action_id: SharedString,
}

#[derive(Clone, Default)]
pub(crate) struct ModulePluginsPaneViewData {
    pub plugins: ModelRc<ModulePluginStatusViewData>,
    pub diagnostics: SharedString,
}

#[derive(Clone, Default)]
pub(crate) struct AnimationEditorPaneViewData {
    pub nodes: ModelRc<ViewTemplateNodeData>,
    pub mode: SharedString,
    pub asset_path: SharedString,
    pub status: SharedString,
    pub selection: SharedString,
    pub current_frame: i32,
    pub timeline_start_frame: i32,
    pub timeline_end_frame: i32,
    pub playback_label: SharedString,
    pub track_items: ModelRc<SharedString>,
    pub parameter_items: ModelRc<SharedString>,
    pub node_items: ModelRc<SharedString>,
    pub state_items: ModelRc<SharedString>,
    pub transition_items: ModelRc<SharedString>,
}

#[derive(Clone)]
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

#[derive(Clone)]
pub(crate) struct ProjectOverviewData {
    pub project_name: SharedString,
    pub project_root: SharedString,
    pub assets_root: SharedString,
    pub library_root: SharedString,
    pub default_scene_uri: SharedString,
    pub catalog_revision: SharedString,
    pub folder_count: SharedString,
    pub asset_count: SharedString,
}

#[derive(Clone)]
pub(crate) struct SceneNodeData {
    pub id: SharedString,
    pub name: SharedString,
    pub depth: i32,
    pub selected: bool,
}

#[derive(Clone)]
pub(crate) struct HostWindowShellData {
    pub project_path: SharedString,
    pub status_secondary: SharedString,
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
    pub shell_min_width_px: f32,
    pub shell_min_height_px: f32,
    pub native_floating_window_mode: bool,
    pub native_floating_window_id: SharedString,
    pub native_window_title: SharedString,
    pub native_window_bounds: FrameRect,
}

#[derive(Clone)]
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

#[derive(Clone)]
pub(crate) struct HostWindowSurfaceMetricsData {
    pub outer_margin_px: f32,
    pub rail_width_px: f32,
    pub top_bar_height_px: f32,
    pub host_bar_height_px: f32,
    pub panel_header_height_px: f32,
    pub document_header_height_px: f32,
}

#[derive(Clone)]
pub(crate) struct HostWindowSurfaceOrchestrationData {
    pub left_rail_width_px: f32,
    pub right_rail_width_px: f32,
    pub left_stack_width_px: f32,
    pub right_stack_width_px: f32,
    pub left_panel_width_px: f32,
    pub right_panel_width_px: f32,
    pub bottom_panel_height_px: f32,
    pub main_content_y_px: f32,
    pub document_zone_x_px: f32,
    pub right_stack_x_px: f32,
    pub bottom_panel_y_px: f32,
}

#[derive(Clone)]
pub(crate) struct HostMenuChromeItemData {
    pub label: SharedString,
    pub shortcut: SharedString,
    pub action_id: SharedString,
    pub enabled: bool,
}

#[derive(Clone)]
pub(crate) struct HostMenuChromeMenuData {
    pub label: SharedString,
    pub popup_width_px: f32,
    pub popup_height_px: f32,
    pub popup_nodes: ModelRc<ViewTemplateNodeData>,
    pub items: ModelRc<HostMenuChromeItemData>,
}

#[derive(Clone)]
pub(crate) struct HostMenuChromeData {
    pub outer_margin_px: f32,
    pub top_bar_height_px: f32,
    pub template_nodes: ModelRc<ViewTemplateNodeData>,
    pub menu_frames: ModelRc<HostChromeControlFrameData>,
    pub save_project_enabled: bool,
    pub undo_enabled: bool,
    pub redo_enabled: bool,
    pub delete_enabled: bool,
    pub preset_names: ModelRc<SharedString>,
    pub active_preset_name: SharedString,
    pub resolved_preset_name: SharedString,
    pub menus: ModelRc<HostMenuChromeMenuData>,
}

#[derive(Clone)]
pub(crate) struct HostPageChromeData {
    pub top_bar_height_px: f32,
    pub host_bar_height_px: f32,
    pub template_nodes: ModelRc<ViewTemplateNodeData>,
    pub tab_row_frame: FrameRect,
    pub project_path_frame: FrameRect,
    pub tab_frames: ModelRc<HostChromeTabData>,
    pub tabs: ModelRc<TabData>,
    pub project_path: SharedString,
}

#[derive(Clone)]
pub(crate) struct HostStatusBarData {
    pub status_bar_frame: FrameRect,
    pub template_nodes: ModelRc<ViewTemplateNodeData>,
    pub status_primary: SharedString,
    pub status_secondary: SharedString,
    pub viewport_label: SharedString,
}

#[derive(Clone)]
pub(crate) struct HostResizeLayerData {
    pub left_splitter_frame: FrameRect,
    pub right_splitter_frame: FrameRect,
    pub bottom_splitter_frame: FrameRect,
}

#[derive(Clone)]
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

#[derive(Clone)]
pub(crate) struct HostSideDockSurfaceData {
    pub region_frame: FrameRect,
    pub surface_key: SharedString,
    pub rail_before_panel: bool,
    pub rail_nodes: ModelRc<ViewTemplateNodeData>,
    pub rail_button_frames: ModelRc<HostChromeControlFrameData>,
    pub rail_active_control_id: SharedString,
    pub header_nodes: ModelRc<ViewTemplateNodeData>,
    pub header_frame: FrameRect,
    pub content_frame: FrameRect,
    pub tab_frames: ModelRc<HostChromeTabData>,
    pub tabs: ModelRc<TabData>,
    pub pane: PaneData,
    pub rail_width_px: f32,
    pub panel_width_px: f32,
    pub panel_header_height_px: f32,
}

#[derive(Clone)]
pub(crate) struct HostDocumentDockSurfaceData {
    pub region_frame: FrameRect,
    pub surface_key: SharedString,
    pub header_nodes: ModelRc<ViewTemplateNodeData>,
    pub header_frame: FrameRect,
    pub subtitle_frame: FrameRect,
    pub content_frame: FrameRect,
    pub tab_frames: ModelRc<HostChromeTabData>,
    pub tabs: ModelRc<TabData>,
    pub pane: PaneData,
    pub header_height_px: f32,
}

#[derive(Clone)]
pub(crate) struct HostBottomDockSurfaceData {
    pub region_frame: FrameRect,
    pub surface_key: SharedString,
    pub header_nodes: ModelRc<ViewTemplateNodeData>,
    pub header_frame: FrameRect,
    pub content_frame: FrameRect,
    pub tab_frames: ModelRc<HostChromeTabData>,
    pub tabs: ModelRc<TabData>,
    pub pane: PaneData,
    pub expanded: bool,
    pub header_height_px: f32,
}

#[derive(Clone)]
pub(crate) struct HostFloatingWindowLayerData {
    pub floating_windows: ModelRc<FloatingWindowData>,
    pub header_height_px: f32,
}

#[derive(Clone)]
pub(crate) struct HostWindowSceneData {
    pub layout: HostWindowLayoutData,
    pub metrics: HostWindowSurfaceMetricsData,
    pub orchestration: HostWindowSurfaceOrchestrationData,
    pub menu_chrome: HostMenuChromeData,
    pub page_chrome: HostPageChromeData,
    pub status_bar: HostStatusBarData,
    pub resize_layer: HostResizeLayerData,
    pub drag_overlay: HostTabDragOverlayData,
    pub left_dock: HostSideDockSurfaceData,
    pub document_dock: HostDocumentDockSurfaceData,
    pub right_dock: HostSideDockSurfaceData,
    pub bottom_dock: HostBottomDockSurfaceData,
    pub floating_layer: HostFloatingWindowLayerData,
}

#[derive(Clone)]
pub(crate) struct HostNativeFloatingWindowSurfaceData {
    pub floating_windows: ModelRc<FloatingWindowData>,
    pub native_floating_window_id: SharedString,
    pub native_window_bounds: FrameRect,
    pub header_height_px: f32,
}
