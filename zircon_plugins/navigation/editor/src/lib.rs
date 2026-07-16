mod bake_panel;
mod capability;
mod extension_ids;
mod operation_command;
mod overlay;
mod plugin;
mod runtime_mirror;

#[cfg(test)]
mod tests;

pub use bake_panel::{
    NavigationBakeAction, NavigationBakeBackend, NavigationBakePanel, NavigationBakePanelBusy,
    NavigationBakePanelController, NavigationBakePhase, NavigationBakeProgress,
    NavigationBakeRequest, NavigationBakeSubmitError,
};
pub use capability::{
    EDITOR_CAPABILITIES, NAVIGATION_AUTHORING_CAPABILITY, NAVIGATION_GIZMOS_CAPABILITY, PLUGIN_ID,
};
pub use extension_ids::{
    NAVIGATION_AGENTS_TEMPLATE_ID, NAVIGATION_AGENTS_VIEW_ID, NAVIGATION_ASSET_TEMPLATE_ID,
    NAVIGATION_ASSET_VIEW_ID, NAVIGATION_AUTHORING_VIEW_ID, NAVIGATION_BAKE_TEMPLATE_ID,
    NAVIGATION_BAKE_VIEW_ID, NAVIGATION_DEBUG_TEMPLATE_ID, NAVIGATION_DEBUG_VIEW_ID,
    NAVIGATION_DRAWER_ID, NAVIGATION_OPEN_NAVMESH_ASSET_OPERATION,
    NAVIGATION_OPEN_SETTINGS_ASSET_OPERATION, NAVIGATION_OPEN_SETTINGS_OPERATION,
    NAVIGATION_SETTINGS_ASSET_TEMPLATE_ID, NAVIGATION_SETTINGS_ASSET_VIEW_ID,
    NAVIGATION_TEMPLATE_ID, NAVIGATION_TOGGLE_GIZMOS_OPERATION, NAV_MESH_AGENT_DRAWER_ID,
    NAV_MESH_MODIFIER_DRAWER_ID, NAV_MESH_OBSTACLE_DRAWER_ID, NAV_MESH_OFF_MESH_LINK_DRAWER_ID,
    NAV_MESH_SURFACE_DRAWER_ID,
};
pub use overlay::{
    build_navigation_overlay, NavigationOverlayController, NavigationOverlayOptions,
    NavigationViewportGizmoSink, NAVIGATION_OVERLAY_MODE_ID, NAVIGATION_OVERLAY_PROVIDER_ID,
};
pub use plugin::{
    editor_capabilities, editor_host_contract_marker, editor_plugin, editor_plugin_descriptor,
    package_manifest, plugin_registration, NavigationEditorPlugin,
};
pub use runtime_mirror::{
    navigation_runtime_event_consumers, NavigationPieFrame, NavigationPieMirror,
    NavigationPieMirrorApply, NAVIGATION_TICK_CONSUMER_ID, NAVIGATION_TICK_EVENT_ID,
    NAVIGATION_TICK_PAYLOAD_SCHEMA,
};
pub use zircon_runtime::core::framework::navigation::NavigationAgentDebugState;
pub use zircon_runtime::core::framework::navigation::{
    NAVIGATION_BAKE_SCENE_OPERATION, NAVIGATION_BAKE_SURFACE_OPERATION,
    NAVIGATION_CLEAR_SURFACE_OPERATION,
};
