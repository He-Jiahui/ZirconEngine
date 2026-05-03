use std::path::{Path, PathBuf};

pub(crate) const UI_HOST_WINDOW_DOCUMENT_ID: &str = "ui.host_window";
pub(crate) const EDITOR_MAIN_FRAME_DOCUMENT_ID: &str = "editor.host.editor_main_frame";
pub(crate) const ACTIVITY_DRAWER_WINDOW_DOCUMENT_ID: &str = "editor.host.activity_drawer_window";
pub(crate) const WORKBENCH_WINDOW_DOCUMENT_ID: &str = "editor.window.workbench";
pub(crate) const ASSET_WINDOW_DOCUMENT_ID: &str = "editor.window.asset";
pub(crate) const UI_COMPONENT_SHOWCASE_WINDOW_DOCUMENT_ID: &str =
    "editor.window.ui_component_showcase";
pub(crate) const UI_LAYOUT_EDITOR_WINDOW_DOCUMENT_ID: &str = "editor.window.ui_layout_editor";
pub(crate) const HOST_DRAWER_SOURCE_DOCUMENT_ID: &str = "workbench.drawer_source";
pub(crate) const FLOATING_WINDOW_SOURCE_DOCUMENT_ID: &str = "floating_window.source";
pub(crate) const SCENE_VIEWPORT_TOOLBAR_DOCUMENT_ID: &str = "scene.viewport_toolbar";
pub(crate) const ASSET_SURFACE_DOCUMENT_ID: &str = "asset.surface_controls";
pub(crate) const WELCOME_SURFACE_DOCUMENT_ID: &str = "startup.welcome_controls";
pub(crate) const INSPECTOR_SURFACE_DOCUMENT_ID: &str = "inspector.surface_controls";
pub(crate) const PANE_SURFACE_DOCUMENT_ID: &str = "pane.surface_controls";
pub(crate) const PANE_CONSOLE_BODY_DOCUMENT_ID: &str = "pane.console.body";
pub(crate) const PANE_INSPECTOR_BODY_DOCUMENT_ID: &str = "pane.inspector.body";
pub(crate) const PANE_HIERARCHY_BODY_DOCUMENT_ID: &str = "pane.hierarchy.body";
pub(crate) const PANE_ANIMATION_SEQUENCE_BODY_DOCUMENT_ID: &str = "pane.animation.sequence.body";
pub(crate) const PANE_ANIMATION_GRAPH_BODY_DOCUMENT_ID: &str = "pane.animation.graph.body";
pub(crate) const PANE_RUNTIME_DIAGNOSTICS_BODY_DOCUMENT_ID: &str = "pane.runtime.diagnostics.body";
pub(crate) const PANE_MODULE_PLUGINS_BODY_DOCUMENT_ID: &str = "pane.module_plugins.body";
pub(crate) const PANE_BUILD_EXPORT_BODY_DOCUMENT_ID: &str = "pane.build_export_desktop.body";
const BUILTIN_HOST_TEMPLATE_ROOT: &str = "/assets/ui/editor/host/";
const BUILTIN_EDITOR_TEMPLATE_ROOT: &str = "/assets/ui/editor/";
const BUILTIN_WINDOW_TEMPLATE_ROOT: &str = "/assets/ui/editor/windows/";

fn builtin_host_template_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(BUILTIN_HOST_TEMPLATE_ROOT.trim_start_matches('/'))
        .join(relative)
}

fn builtin_window_template_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(BUILTIN_WINDOW_TEMPLATE_ROOT.trim_start_matches('/'))
        .join(relative)
}

fn builtin_editor_template_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(BUILTIN_EDITOR_TEMPLATE_ROOT.trim_start_matches('/'))
        .join(relative)
}

pub(crate) fn builtin_template_documents() -> [(&'static str, PathBuf); 22] {
    [
        (
            EDITOR_MAIN_FRAME_DOCUMENT_ID,
            builtin_host_template_path("editor_main_frame.ui.toml"),
        ),
        (
            ACTIVITY_DRAWER_WINDOW_DOCUMENT_ID,
            builtin_host_template_path("activity_drawer_window.ui.toml"),
        ),
        (
            WORKBENCH_WINDOW_DOCUMENT_ID,
            builtin_window_template_path("workbench_window.ui.toml"),
        ),
        (
            ASSET_WINDOW_DOCUMENT_ID,
            builtin_window_template_path("asset_window.ui.toml"),
        ),
        (
            UI_LAYOUT_EDITOR_WINDOW_DOCUMENT_ID,
            builtin_window_template_path("ui_layout_editor_window.ui.toml"),
        ),
        (
            UI_COMPONENT_SHOWCASE_WINDOW_DOCUMENT_ID,
            builtin_editor_template_path("component_showcase.ui.toml"),
        ),
        (
            UI_HOST_WINDOW_DOCUMENT_ID,
            builtin_host_template_path("workbench_shell.ui.toml"),
        ),
        (
            HOST_DRAWER_SOURCE_DOCUMENT_ID,
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
        (
            PANE_CONSOLE_BODY_DOCUMENT_ID,
            builtin_host_template_path("console_body.ui.toml"),
        ),
        (
            PANE_INSPECTOR_BODY_DOCUMENT_ID,
            builtin_host_template_path("inspector_body.ui.toml"),
        ),
        (
            PANE_HIERARCHY_BODY_DOCUMENT_ID,
            builtin_host_template_path("hierarchy_body.ui.toml"),
        ),
        (
            PANE_ANIMATION_SEQUENCE_BODY_DOCUMENT_ID,
            builtin_host_template_path("animation_sequence_body.ui.toml"),
        ),
        (
            PANE_ANIMATION_GRAPH_BODY_DOCUMENT_ID,
            builtin_host_template_path("animation_graph_body.ui.toml"),
        ),
        (
            PANE_RUNTIME_DIAGNOSTICS_BODY_DOCUMENT_ID,
            builtin_host_template_path("runtime_diagnostics_body.ui.toml"),
        ),
        (
            PANE_MODULE_PLUGINS_BODY_DOCUMENT_ID,
            builtin_host_template_path("module_plugins_body.ui.toml"),
        ),
        (
            PANE_BUILD_EXPORT_BODY_DOCUMENT_ID,
            builtin_host_template_path("build_export_desktop_body.ui.toml"),
        ),
    ]
}
