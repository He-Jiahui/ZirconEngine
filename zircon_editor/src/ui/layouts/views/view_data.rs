use slint::{Image, ModelRc, SharedString};

#[derive(Clone)]
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

#[derive(Clone)]
pub(crate) struct NewProjectFormData {
    pub project_name: SharedString,
    pub location: SharedString,
    pub project_path_preview: SharedString,
    pub template_label: SharedString,
    pub validation_message: SharedString,
    pub can_create: bool,
    pub can_open_existing: bool,
    pub browse_supported: bool,
}

#[derive(Clone)]
pub(crate) struct WelcomePaneData {
    pub nodes: ModelRc<ViewTemplateNodeData>,
    pub title: SharedString,
    pub subtitle: SharedString,
    pub status_message: SharedString,
    pub form: NewProjectFormData,
}

#[derive(Clone)]
pub(crate) struct RecentProjectData {
    pub display_name: SharedString,
    pub path: SharedString,
    pub last_opened_label: SharedString,
    pub status_label: SharedString,
    pub invalid: bool,
}

#[derive(Clone)]
pub(crate) struct AssetFolderData {
    pub id: SharedString,
    pub name: SharedString,
    pub count: i32,
    pub depth: i32,
    pub selected: bool,
}

#[derive(Clone)]
pub(crate) struct AssetItemData {
    pub uuid: SharedString,
    pub locator: SharedString,
    pub name: SharedString,
    pub file_name: SharedString,
    pub kind: SharedString,
    pub extension: SharedString,
    pub dirty: bool,
    pub has_error: bool,
    pub has_preview: bool,
    pub state: SharedString,
    pub revision: SharedString,
    pub selected: bool,
    pub preview: Image,
}

#[derive(Clone)]
pub(crate) struct AssetReferenceData {
    pub uuid: SharedString,
    pub locator: SharedString,
    pub name: SharedString,
    pub kind: SharedString,
    pub known_project_asset: bool,
}

#[derive(Clone)]
pub(crate) struct AssetSelectionData {
    pub uuid: SharedString,
    pub name: SharedString,
    pub locator: SharedString,
    pub kind: SharedString,
    pub meta_path: SharedString,
    pub adapter_key: SharedString,
    pub state: SharedString,
    pub revision: SharedString,
    pub diagnostics: SharedString,
    pub has_preview: bool,
    pub preview: Image,
}

#[derive(Clone)]
pub(crate) struct WelcomePresentation {
    pub pane: WelcomePaneData,
    pub recent_projects: ModelRc<RecentProjectData>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ViewTemplateFrameData {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ViewTemplateNodeData {
    pub node_id: SharedString,
    pub control_id: SharedString,
    pub role: SharedString,
    pub text: SharedString,
    pub dispatch_kind: SharedString,
    pub action_id: SharedString,
    pub surface_variant: SharedString,
    pub text_tone: SharedString,
    pub button_variant: SharedString,
    pub font_size: f32,
    pub font_weight: i32,
    pub text_align: SharedString,
    pub overflow: SharedString,
    pub corner_radius: f32,
    pub border_width: f32,
    pub frame: ViewTemplateFrameData,
}
