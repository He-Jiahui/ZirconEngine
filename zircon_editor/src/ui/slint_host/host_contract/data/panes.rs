use slint::{ModelRc, SharedString};

use super::{TemplatePaneNodeData, UiAssetEditorPaneData};

#[derive(Clone, Default)]
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

#[derive(Clone, Default)]
pub(crate) struct SceneNodeData {
    pub id: SharedString,
    pub name: SharedString,
    pub depth: i32,
    pub selected: bool,
}

#[derive(Clone, Default)]
pub(crate) struct HierarchyPaneData {
    pub nodes: ModelRc<TemplatePaneNodeData>,
    pub hierarchy_nodes: ModelRc<SceneNodeData>,
}

#[derive(Clone, Default)]
pub(crate) struct AnimationEditorPaneData {
    pub nodes: ModelRc<TemplatePaneNodeData>,
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

#[derive(Clone, Default)]
pub(crate) struct InspectorPaneData {
    pub nodes: ModelRc<TemplatePaneNodeData>,
    pub info: SharedString,
    pub inspector_name: SharedString,
    pub inspector_parent: SharedString,
    pub inspector_x: SharedString,
    pub inspector_y: SharedString,
    pub inspector_z: SharedString,
    pub delete_enabled: bool,
}

#[derive(Clone, Default)]
pub(crate) struct SceneViewportChromeData {
    pub tool: SharedString,
    pub transform_space: SharedString,
    pub projection_mode: SharedString,
    pub view_orientation: SharedString,
    pub display_mode: SharedString,
    pub grid_mode: SharedString,
    pub gizmos_enabled: bool,
    pub preview_lighting: bool,
    pub preview_skybox: bool,
    pub translate_snap: f32,
    pub rotate_snap_deg: f32,
    pub scale_snap: f32,
    pub translate_snap_label: SharedString,
    pub rotate_snap_label: SharedString,
    pub scale_snap_label: SharedString,
}

#[derive(Clone, Default)]
pub(crate) struct ConsolePaneData {
    pub nodes: ModelRc<TemplatePaneNodeData>,
    pub status_text: SharedString,
}

#[derive(Clone, Default)]
pub(crate) struct AssetsActivityPaneData {
    pub nodes: ModelRc<TemplatePaneNodeData>,
}

#[derive(Clone, Default)]
pub(crate) struct AssetBrowserPaneData {
    pub nodes: ModelRc<TemplatePaneNodeData>,
}

#[derive(Clone, Default)]
pub(crate) struct ProjectOverviewPaneData {
    pub nodes: ModelRc<TemplatePaneNodeData>,
}

#[derive(Clone, Default)]
pub(crate) struct ModulePluginStatusData {
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
    pub diagnostics: SharedString,
}

#[derive(Clone, Default)]
pub(crate) struct ModulePluginsPaneData {
    pub plugins: ModelRc<ModulePluginStatusData>,
    pub diagnostics: SharedString,
}

#[derive(Clone, Default)]
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
    pub hierarchy: HierarchyPaneData,
    pub inspector: InspectorPaneData,
    pub console: ConsolePaneData,
    pub assets_activity: AssetsActivityPaneData,
    pub asset_browser: AssetBrowserPaneData,
    pub project_overview: ProjectOverviewPaneData,
    pub module_plugins: ModulePluginsPaneData,
    pub ui_asset: UiAssetEditorPaneData,
    pub animation: AnimationEditorPaneData,
}
