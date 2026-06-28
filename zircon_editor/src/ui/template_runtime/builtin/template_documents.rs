use std::path::PathBuf;

use zircon_runtime::asset::runtime_asset_path_with_dev_asset_root;
use zircon_runtime::diagnostic_log::{
    diagnostic_log_allows, write_diagnostic_log, DiagnosticLogLevel,
};

pub(crate) const UI_HOST_WINDOW_DOCUMENT_ID: &str = "ui.host_window";
pub(crate) const EDITOR_MAIN_FRAME_DOCUMENT_ID: &str = "editor.host.editor_main_frame";
pub(crate) const WORKBENCH_WINDOW_DOCUMENT_ID: &str = "editor.window.workbench";
pub(crate) const ASSET_WINDOW_DOCUMENT_ID: &str = "editor.window.asset";
pub(crate) const UI_COMPONENT_SHOWCASE_WINDOW_DOCUMENT_ID: &str =
    "editor.window.ui_component_showcase";
pub(crate) const MATERIAL_DEMO_WINDOW_DOCUMENT_ID: &str = "editor.window.material_demo";
pub(crate) const MATERIAL_COMPONENT_LAB_WINDOW_DOCUMENT_ID: &str =
    "editor.window.material_component_lab";
pub(crate) const UI_LAYOUT_EDITOR_WINDOW_DOCUMENT_ID: &str = "editor.window.ui_layout_editor";
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
pub(crate) const PANE_PERFORMANCE_TIMELINE_BODY_DOCUMENT_ID: &str =
    "pane.performance.timeline.body";
pub(crate) const PANE_MODULE_PLUGINS_BODY_DOCUMENT_ID: &str = "pane.module_plugins.body";
pub(crate) const PANE_BUILD_EXPORT_BODY_DOCUMENT_ID: &str = "pane.build_export_desktop.body";
pub(crate) const PANE_GENERATED_BOTTOM_BODY_DOCUMENT_ID: &str = "pane.generated_bottom.body";
const BUILTIN_HOST_TEMPLATE_ROOT: &str = "/assets/ui/editor/host/";
const BUILTIN_EDITOR_TEMPLATE_ROOT: &str = "/assets/ui/editor/";
const BUILTIN_WINDOW_TEMPLATE_ROOT: &str = "/assets/ui/editor/windows/";

fn builtin_host_template_path(relative: &str) -> PathBuf {
    editor_runtime_asset_path(BUILTIN_HOST_TEMPLATE_ROOT).join(relative)
}

fn builtin_window_template_path(relative: &str) -> PathBuf {
    editor_runtime_asset_path(BUILTIN_WINDOW_TEMPLATE_ROOT).join(relative)
}

fn builtin_editor_template_path(relative: &str) -> PathBuf {
    editor_runtime_asset_path(BUILTIN_EDITOR_TEMPLATE_ROOT).join(relative)
}

fn editor_runtime_asset_path(relative: &str) -> PathBuf {
    runtime_asset_path_with_dev_asset_root(relative, editor_dev_asset_root())
}

fn editor_dev_asset_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")
}

pub(crate) fn builtin_template_documents() -> [(&'static str, PathBuf); 24] {
    let documents = [
        (
            EDITOR_MAIN_FRAME_DOCUMENT_ID,
            builtin_host_template_path("editor_main_frame.zui"),
        ),
        (
            WORKBENCH_WINDOW_DOCUMENT_ID,
            builtin_window_template_path("workbench_window.zui"),
        ),
        (
            ASSET_WINDOW_DOCUMENT_ID,
            builtin_window_template_path("asset_window.zui"),
        ),
        (
            UI_LAYOUT_EDITOR_WINDOW_DOCUMENT_ID,
            builtin_window_template_path("ui_layout_editor_window.zui"),
        ),
        (
            UI_COMPONENT_SHOWCASE_WINDOW_DOCUMENT_ID,
            builtin_editor_template_path("component_showcase.zui"),
        ),
        (
            MATERIAL_DEMO_WINDOW_DOCUMENT_ID,
            builtin_editor_template_path("material_demo_window.zui"),
        ),
        (
            MATERIAL_COMPONENT_LAB_WINDOW_DOCUMENT_ID,
            builtin_editor_template_path("material_component_lab.zui"),
        ),
        (
            UI_HOST_WINDOW_DOCUMENT_ID,
            builtin_host_template_path("workbench_shell.zui"),
        ),
        (
            FLOATING_WINDOW_SOURCE_DOCUMENT_ID,
            builtin_host_template_path("floating_window_source.zui"),
        ),
        (
            SCENE_VIEWPORT_TOOLBAR_DOCUMENT_ID,
            builtin_host_template_path("scene_viewport_toolbar.zui"),
        ),
        (
            ASSET_SURFACE_DOCUMENT_ID,
            builtin_host_template_path("asset_surface_controls.zui"),
        ),
        (
            WELCOME_SURFACE_DOCUMENT_ID,
            builtin_host_template_path("startup_welcome_controls.zui"),
        ),
        (
            INSPECTOR_SURFACE_DOCUMENT_ID,
            builtin_host_template_path("inspector_surface_controls.zui"),
        ),
        (
            PANE_SURFACE_DOCUMENT_ID,
            builtin_host_template_path("pane_surface_controls.zui"),
        ),
        (
            PANE_CONSOLE_BODY_DOCUMENT_ID,
            builtin_host_template_path("console_body.zui"),
        ),
        (
            PANE_INSPECTOR_BODY_DOCUMENT_ID,
            builtin_host_template_path("inspector_body.zui"),
        ),
        (
            PANE_HIERARCHY_BODY_DOCUMENT_ID,
            builtin_host_template_path("hierarchy_body.zui"),
        ),
        (
            PANE_ANIMATION_SEQUENCE_BODY_DOCUMENT_ID,
            builtin_host_template_path("animation_sequence_body.zui"),
        ),
        (
            PANE_ANIMATION_GRAPH_BODY_DOCUMENT_ID,
            builtin_host_template_path("animation_graph_body.zui"),
        ),
        (
            PANE_RUNTIME_DIAGNOSTICS_BODY_DOCUMENT_ID,
            builtin_host_template_path("runtime_diagnostics_body.zui"),
        ),
        (
            PANE_PERFORMANCE_TIMELINE_BODY_DOCUMENT_ID,
            builtin_host_template_path("performance_timeline_body.zui"),
        ),
        (
            PANE_MODULE_PLUGINS_BODY_DOCUMENT_ID,
            builtin_host_template_path("module_plugins_body.zui"),
        ),
        (
            PANE_BUILD_EXPORT_BODY_DOCUMENT_ID,
            builtin_host_template_path("build_export_desktop_body.zui"),
        ),
        (
            PANE_GENERATED_BOTTOM_BODY_DOCUMENT_ID,
            builtin_host_template_path("generated_bottom_body.zui"),
        ),
    ];

    if diagnostic_log_allows(DiagnosticLogLevel::Verbose) {
        for (document_id, path) in &documents {
            write_diagnostic_log(
                "editor_builtin_templates",
                format!(
                    "document id={} path={} exists={}",
                    document_id,
                    path.display(),
                    path.exists()
                ),
            );
        }
    }

    documents
}
