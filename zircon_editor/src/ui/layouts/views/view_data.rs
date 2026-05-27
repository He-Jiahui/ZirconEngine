use std::fmt;

use crate::ui::retained_host::primitives::{Image, ModelRc, SharedString};
use zircon_runtime_interface::ui::style::ResolvedButtonStyle;

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

#[derive(Clone)]
pub(crate) struct ViewTemplateNodeData {
    pub node_id: SharedString,
    pub control_id: SharedString,
    pub role: SharedString,
    pub text: SharedString,
    pub component_role: SharedString,
    pub component_variant: SharedString,
    pub value_text: SharedString,
    pub value_number: f32,
    pub value_percent: f32,
    pub dispatch_kind: SharedString,
    pub action_id: SharedString,
    pub binding_id: SharedString,
    pub edit_action_id: SharedString,
    pub commit_action_id: SharedString,
    pub surface_variant: SharedString,
    pub text_tone: SharedString,
    pub button_variant: SharedString,
    pub button_style: ResolvedButtonStyle,
    pub font_size: f32,
    pub font_weight: i32,
    pub text_align: SharedString,
    pub overflow: SharedString,
    pub corner_radius: f32,
    pub border_width: f32,
    pub z_index: i32,
    pub transition_kind: SharedString,
    pub transition_in: bool,
    pub transition_entered: bool,
    pub transition_progress: f32,
    pub transition_duration_ms: i32,
    pub transition_easing: SharedString,
    pub transition_direction: SharedString,
    pub selected: bool,
    pub popup_open: bool,
    pub focused: bool,
    pub hovered: bool,
    pub pressed: bool,
    pub disabled: bool,
    pub media_source: SharedString,
    pub icon_name: SharedString,
    pub has_preview_image: bool,
    pub preview_image: Image,
    pub frame: ViewTemplateFrameData,
}

impl fmt::Debug for ViewTemplateNodeData {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let preview_size = self.preview_image.size();
        formatter
            .debug_struct("ViewTemplateNodeData")
            .field("node_id", &self.node_id)
            .field("control_id", &self.control_id)
            .field("role", &self.role)
            .field("text", &self.text)
            .field("component_role", &self.component_role)
            .field("component_variant", &self.component_variant)
            .field("value_text", &self.value_text)
            .field("value_number", &self.value_number)
            .field("value_percent", &self.value_percent)
            .field("dispatch_kind", &self.dispatch_kind)
            .field("action_id", &self.action_id)
            .field("binding_id", &self.binding_id)
            .field("edit_action_id", &self.edit_action_id)
            .field("commit_action_id", &self.commit_action_id)
            .field("surface_variant", &self.surface_variant)
            .field("text_tone", &self.text_tone)
            .field("button_variant", &self.button_variant)
            .field("button_style", &self.button_style)
            .field("font_size", &self.font_size)
            .field("font_weight", &self.font_weight)
            .field("text_align", &self.text_align)
            .field("overflow", &self.overflow)
            .field("corner_radius", &self.corner_radius)
            .field("border_width", &self.border_width)
            .field("z_index", &self.z_index)
            .field("transition_kind", &self.transition_kind)
            .field("transition_in", &self.transition_in)
            .field("transition_entered", &self.transition_entered)
            .field("transition_progress", &self.transition_progress)
            .field("transition_duration_ms", &self.transition_duration_ms)
            .field("transition_easing", &self.transition_easing)
            .field("transition_direction", &self.transition_direction)
            .field("selected", &self.selected)
            .field("popup_open", &self.popup_open)
            .field("focused", &self.focused)
            .field("hovered", &self.hovered)
            .field("pressed", &self.pressed)
            .field("disabled", &self.disabled)
            .field("media_source", &self.media_source)
            .field("icon_name", &self.icon_name)
            .field("has_preview_image", &self.has_preview_image)
            .field(
                "preview_image_size",
                &(preview_size.width, preview_size.height),
            )
            .field("frame", &self.frame)
            .finish()
    }
}

impl PartialEq for ViewTemplateNodeData {
    fn eq(&self, other: &Self) -> bool {
        let preview_size = self.preview_image.size();
        let other_preview_size = other.preview_image.size();
        self.node_id == other.node_id
            && self.control_id == other.control_id
            && self.role == other.role
            && self.text == other.text
            && self.component_role == other.component_role
            && self.component_variant == other.component_variant
            && self.value_text == other.value_text
            && self.value_number == other.value_number
            && self.value_percent == other.value_percent
            && self.dispatch_kind == other.dispatch_kind
            && self.action_id == other.action_id
            && self.binding_id == other.binding_id
            && self.edit_action_id == other.edit_action_id
            && self.commit_action_id == other.commit_action_id
            && self.surface_variant == other.surface_variant
            && self.text_tone == other.text_tone
            && self.button_variant == other.button_variant
            && self.button_style == other.button_style
            && self.font_size == other.font_size
            && self.font_weight == other.font_weight
            && self.text_align == other.text_align
            && self.overflow == other.overflow
            && self.corner_radius == other.corner_radius
            && self.border_width == other.border_width
            && self.z_index == other.z_index
            && self.transition_kind == other.transition_kind
            && self.transition_in == other.transition_in
            && self.transition_entered == other.transition_entered
            && self.transition_progress == other.transition_progress
            && self.transition_duration_ms == other.transition_duration_ms
            && self.transition_easing == other.transition_easing
            && self.transition_direction == other.transition_direction
            && self.selected == other.selected
            && self.popup_open == other.popup_open
            && self.focused == other.focused
            && self.hovered == other.hovered
            && self.pressed == other.pressed
            && self.disabled == other.disabled
            && self.media_source == other.media_source
            && self.icon_name == other.icon_name
            && self.has_preview_image == other.has_preview_image
            && preview_size.width == other_preview_size.width
            && preview_size.height == other_preview_size.height
            && self.frame == other.frame
    }
}

impl Default for ViewTemplateNodeData {
    fn default() -> Self {
        Self {
            node_id: SharedString::default(),
            control_id: SharedString::default(),
            role: SharedString::default(),
            text: SharedString::default(),
            component_role: SharedString::default(),
            component_variant: SharedString::default(),
            value_text: SharedString::default(),
            value_number: 0.0,
            value_percent: 0.0,
            dispatch_kind: SharedString::default(),
            action_id: SharedString::default(),
            binding_id: SharedString::default(),
            edit_action_id: SharedString::default(),
            commit_action_id: SharedString::default(),
            surface_variant: SharedString::default(),
            text_tone: SharedString::default(),
            button_variant: SharedString::default(),
            button_style: ResolvedButtonStyle::default(),
            font_size: 0.0,
            font_weight: 0,
            text_align: SharedString::default(),
            overflow: SharedString::default(),
            corner_radius: 0.0,
            border_width: 0.0,
            z_index: 0,
            transition_kind: SharedString::default(),
            transition_in: true,
            transition_entered: true,
            transition_progress: 1.0,
            transition_duration_ms: 0,
            transition_easing: SharedString::default(),
            transition_direction: SharedString::default(),
            selected: false,
            popup_open: false,
            focused: false,
            hovered: false,
            pressed: false,
            disabled: false,
            media_source: SharedString::default(),
            icon_name: SharedString::default(),
            has_preview_image: false,
            preview_image: Image::default(),
            frame: ViewTemplateFrameData::default(),
        }
    }
}
