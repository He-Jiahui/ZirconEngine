use std::path::{Path, PathBuf};

pub(crate) const UI_HOST_WINDOW_DOCUMENT_ID: &str = "ui.host_window";
pub(crate) const WORKBENCH_SHELL_DOCUMENT_ID: &str = "workbench.shell";
pub(crate) const WORKBENCH_DRAWER_SOURCE_DOCUMENT_ID: &str = "workbench.drawer_source";
pub(crate) const FLOATING_WINDOW_SOURCE_DOCUMENT_ID: &str = "floating_window.source";
pub(crate) const SCENE_VIEWPORT_TOOLBAR_DOCUMENT_ID: &str = "scene.viewport_toolbar";
pub(crate) const ASSET_SURFACE_DOCUMENT_ID: &str = "asset.surface_controls";
pub(crate) const WELCOME_SURFACE_DOCUMENT_ID: &str = "startup.welcome_controls";
pub(crate) const INSPECTOR_SURFACE_DOCUMENT_ID: &str = "inspector.surface_controls";
pub(crate) const PANE_SURFACE_DOCUMENT_ID: &str = "pane.surface_controls";
const BUILTIN_HOST_TEMPLATE_ROOT: &str = "/assets/ui/editor/host/";

fn builtin_host_template_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(BUILTIN_HOST_TEMPLATE_ROOT.trim_start_matches('/'))
        .join(relative)
}

pub(crate) fn builtin_template_documents() -> [(&'static str, PathBuf); 9] {
    [
        (
            UI_HOST_WINDOW_DOCUMENT_ID,
            builtin_host_template_path("workbench_shell.ui.toml"),
        ),
        (
            WORKBENCH_SHELL_DOCUMENT_ID,
            builtin_host_template_path("workbench_shell.ui.toml"),
        ),
        (
            WORKBENCH_DRAWER_SOURCE_DOCUMENT_ID,
            builtin_host_template_path("workbench_drawer_source.ui.toml"),
        ),
        (
            FLOATING_WINDOW_SOURCE_DOCUMENT_ID,
            builtin_host_template_path("floating_window_source.ui.toml"),
        ),
        (
            SCENE_VIEWPORT_TOOLBAR_DOCUMENT_ID,
            builtin_host_template_path("scene_viewport_toolbar.ui.toml"),
        ),
        (
            ASSET_SURFACE_DOCUMENT_ID,
            builtin_host_template_path("asset_surface_controls.ui.toml"),
        ),
        (
            WELCOME_SURFACE_DOCUMENT_ID,
            builtin_host_template_path("startup_welcome_controls.ui.toml"),
        ),
        (
            INSPECTOR_SURFACE_DOCUMENT_ID,
            builtin_host_template_path("inspector_surface_controls.ui.toml"),
        ),
        (
            PANE_SURFACE_DOCUMENT_ID,
            builtin_host_template_path("pane_surface_controls.ui.toml"),
        ),
    ]
}
