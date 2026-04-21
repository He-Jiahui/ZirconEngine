pub(crate) const UI_HOST_WINDOW_DOCUMENT_ID: &str = "ui.host_window";
pub(crate) const WORKBENCH_SHELL_DOCUMENT_ID: &str = "workbench.shell";
pub(crate) const WORKBENCH_DRAWER_SOURCE_DOCUMENT_ID: &str = "workbench.drawer_source";
pub(crate) const FLOATING_WINDOW_SOURCE_DOCUMENT_ID: &str = "floating_window.source";
pub(crate) const SCENE_VIEWPORT_TOOLBAR_DOCUMENT_ID: &str = "scene.viewport_toolbar";
pub(crate) const ASSET_SURFACE_DOCUMENT_ID: &str = "asset.surface_controls";
pub(crate) const WELCOME_SURFACE_DOCUMENT_ID: &str = "startup.welcome_controls";
pub(crate) const INSPECTOR_SURFACE_DOCUMENT_ID: &str = "inspector.surface_controls";
pub(crate) const PANE_SURFACE_DOCUMENT_ID: &str = "pane.surface_controls";

macro_rules! builtin_host_template {
    ($relative:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/ui/editor/host/",
            $relative
        ))
    };
}

const UI_HOST_WINDOW_TEMPLATE: &str = builtin_host_template!("workbench_shell.ui.toml");
const WORKBENCH_DRAWER_SOURCE_TEMPLATE: &str =
    builtin_host_template!("workbench_drawer_source.ui.toml");
const FLOATING_WINDOW_SOURCE_TEMPLATE: &str =
    builtin_host_template!("floating_window_source.ui.toml");
const SCENE_VIEWPORT_TOOLBAR_TEMPLATE: &str =
    builtin_host_template!("scene_viewport_toolbar.ui.toml");
const ASSET_SURFACE_TEMPLATE: &str = builtin_host_template!("asset_surface_controls.ui.toml");
const WELCOME_SURFACE_TEMPLATE: &str = builtin_host_template!("startup_welcome_controls.ui.toml");
const INSPECTOR_SURFACE_TEMPLATE: &str =
    builtin_host_template!("inspector_surface_controls.ui.toml");
const PANE_SURFACE_TEMPLATE: &str = builtin_host_template!("pane_surface_controls.ui.toml");

pub(crate) fn builtin_template_documents() -> [(&'static str, &'static str); 9] {
    [
        (UI_HOST_WINDOW_DOCUMENT_ID, UI_HOST_WINDOW_TEMPLATE),
        (WORKBENCH_SHELL_DOCUMENT_ID, UI_HOST_WINDOW_TEMPLATE),
        (
            WORKBENCH_DRAWER_SOURCE_DOCUMENT_ID,
            WORKBENCH_DRAWER_SOURCE_TEMPLATE,
        ),
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
