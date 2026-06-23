pub const PLUGIN_ID: &str = zircon_plugin_navigation_runtime::PLUGIN_ID;
pub const NAVIGATION_AUTHORING_CAPABILITY: &str = "editor.extension.navigation_authoring";
pub const NAVIGATION_GIZMOS_CAPABILITY: &str = "editor.extension.navigation_gizmos";

pub const EDITOR_CAPABILITIES: &[&str] = &[
    NAVIGATION_AUTHORING_CAPABILITY,
    NAVIGATION_GIZMOS_CAPABILITY,
];
