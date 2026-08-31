mod animation_editor;
mod asset_browser;
mod asset_kind_filter;
mod asset_reference_rows;
mod assets_activity;
mod console;
mod hierarchy;
mod inspector;
mod project_overview;
mod view_data;
mod view_projection;
mod viewport_chrome;
mod welcome;
mod welcome_presentation;

pub(crate) use animation_editor::animation_editor_pane_nodes;
pub(crate) use asset_browser::{asset_browser_pane_data, asset_browser_pane_nodes};
pub(crate) use asset_kind_filter::{
    asset_kind_filter_identity, asset_kind_filter_is_supported, asset_kind_filter_options,
    ASSETS_ACTIVITY_KIND_FILTER_CONTROL_ID, ASSET_BROWSER_KIND_FILTER_CONTROL_ID,
    ASSET_KIND_FILTER_OPTIONS,
};
pub(crate) use assets_activity::assets_activity_pane_data;
pub(crate) use console::console_pane_nodes;
pub(crate) use hierarchy::hierarchy_pane_nodes;
pub(crate) use inspector::inspector_pane_nodes;
pub(crate) use project_overview::{project_overview_data, project_overview_pane_data};
pub(crate) use view_data::{
    NewProjectFormData, RecentProjectData, SceneViewportChromeData, WelcomePaneData,
    WelcomePresentation,
};
pub(crate) use view_data::{ViewTemplateFrameData, ViewTemplateNodeData};
#[cfg(test)]
pub(crate) use view_projection::clear_view_template_projection_caches_for_tests;
pub(crate) use view_projection::{
    build_view_template_node_projection, build_view_template_node_projection_with_patches,
    compose_view_template_node_model, default_transition_duration_ms, default_transition_easing,
    preferred_binding_id, resolve_commit_action_id, resolve_component_role,
    resolve_component_variant, resolve_edit_action_id, resolve_node_popup_open,
    resolve_node_value_number, resolve_node_value_percent, resolve_node_value_text,
    resolve_transition_in, resolve_transition_kind, resolve_transition_progress,
    resolve_visual_assets, view_template_resource_generation, ViewTemplateNodePatch,
    ViewTemplateResourceGeneration,
};
pub(crate) use viewport_chrome::{blank_viewport_chrome, scene_viewport_chrome};
pub(crate) use welcome::welcome_pane_nodes;
pub(crate) use welcome_presentation::welcome_presentation;
