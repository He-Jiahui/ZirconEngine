#[path = "animation_editor.rs"]
mod animation_editor;
#[path = "asset_browser.rs"]
mod asset_browser;
#[path = "asset_kind_filter.rs"]
mod asset_kind_filter;
#[path = "asset_reference_rows.rs"]
mod asset_reference_rows;
#[path = "assets_activity.rs"]
mod assets_activity;
#[path = "console.rs"]
mod console;
#[path = "hierarchy.rs"]
mod hierarchy;
#[path = "inspector.rs"]
mod inspector;
#[path = "project_overview.rs"]
mod project_overview;
#[path = "view_data.rs"]
mod view_data;
#[path = "view_projection.rs"]
mod view_projection;
#[path = "viewport_chrome.rs"]
mod viewport_chrome;
#[path = "welcome.rs"]
mod welcome;
#[path = "welcome_presentation.rs"]
mod welcome_presentation;

pub(crate) use animation_editor::{animation_graph_pane_nodes, animation_sequence_pane_nodes};
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
    resolve_visual_assets, view_template_resource_generation, UnrealDockHeaderSource,
    ViewTemplateNodePatch, ViewTemplateResourceGeneration,
};
pub(crate) use viewport_chrome::{blank_viewport_chrome, scene_viewport_chrome};
pub(crate) use welcome::welcome_pane_nodes;
pub(crate) use welcome_presentation::welcome_presentation;
