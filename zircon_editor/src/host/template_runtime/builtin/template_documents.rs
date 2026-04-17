pub(crate) const WORKBENCH_SHELL_DOCUMENT_ID: &str = "workbench.shell";
pub(crate) const FLOATING_WINDOW_SOURCE_DOCUMENT_ID: &str = "floating_window.source";
pub(crate) const SCENE_VIEWPORT_TOOLBAR_DOCUMENT_ID: &str = "scene.viewport_toolbar";
pub(crate) const ASSET_SURFACE_DOCUMENT_ID: &str = "asset.surface_controls";
pub(crate) const WELCOME_SURFACE_DOCUMENT_ID: &str = "startup.welcome_controls";
pub(crate) const INSPECTOR_SURFACE_DOCUMENT_ID: &str = "inspector.surface_controls";
pub(crate) const PANE_SURFACE_DOCUMENT_ID: &str = "pane.surface_controls";

const WORKBENCH_SHELL_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/ui/templates/workbench_shell.toml"
));
const FLOATING_WINDOW_SOURCE_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/ui/templates/floating_window_source.toml"
));
const SCENE_VIEWPORT_TOOLBAR_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/ui/templates/scene_viewport_toolbar.toml"
));
const ASSET_SURFACE_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/ui/templates/asset_surface_controls.toml"
));
const WELCOME_SURFACE_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/ui/templates/startup_welcome_controls.toml"
));
const INSPECTOR_SURFACE_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/ui/templates/inspector_surface_controls.toml"
));
const PANE_SURFACE_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/ui/templates/pane_surface_controls.toml"
));

pub(crate) fn builtin_template_documents() -> [(&'static str, &'static str); 7] {
    [
        (WORKBENCH_SHELL_DOCUMENT_ID, WORKBENCH_SHELL_TEMPLATE),
        (
            FLOATING_WINDOW_SOURCE_DOCUMENT_ID,
            FLOATING_WINDOW_SOURCE_TEMPLATE,
        ),
        (
            SCENE_VIEWPORT_TOOLBAR_DOCUMENT_ID,
            SCENE_VIEWPORT_TOOLBAR_TEMPLATE,
        ),
        (ASSET_SURFACE_DOCUMENT_ID, ASSET_SURFACE_TEMPLATE),
        (WELCOME_SURFACE_DOCUMENT_ID, WELCOME_SURFACE_TEMPLATE),
        (INSPECTOR_SURFACE_DOCUMENT_ID, INSPECTOR_SURFACE_TEMPLATE),
        (PANE_SURFACE_DOCUMENT_ID, PANE_SURFACE_TEMPLATE),
    ]
}
