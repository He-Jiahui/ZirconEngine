#[path = "asset_surface/mod.rs"]
mod asset_surface;
#[path = "floating_window_source/mod.rs"]
mod floating_window_source;
#[path = "inspector_surface/mod.rs"]
mod inspector_surface;
#[path = "pane_surface/mod.rs"]
mod pane_surface;
#[path = "popup_primitives.rs"]
mod popup_primitives;
#[path = "projection_support.rs"]
mod projection_support;
#[path = "viewport_toolbar/mod.rs"]
mod viewport_toolbar;
#[path = "virtual_rows.rs"]
mod virtual_rows;
#[path = "welcome_surface/mod.rs"]
mod welcome_surface;
#[path = "workbench/mod.rs"]
mod workbench;

use crate::ui::template_runtime::builtin::{
    PANE_CONSOLE_BODY_DOCUMENT_ID, PANE_HIERARCHY_BODY_DOCUMENT_ID, PANE_INSPECTOR_BODY_DOCUMENT_ID,
};
use crate::ui::template_runtime::{
    EditorUiHostRuntime, EditorUiHostRuntimeError, UNREAL_EDITOR_WINDOW_DOCUMENT_ID,
};

use super::constants::{
    BUILTIN_FLOATING_WINDOW_SOURCE_DOCUMENT_ID, BUILTIN_INSPECTOR_SURFACE_DOCUMENT_ID,
    BUILTIN_PANE_SURFACE_DOCUMENT_ID, BUILTIN_VIEWPORT_TOOLBAR_DOCUMENT_ID,
    UNREAL_EDITOR_SHELL_DOCUMENT_ID,
};

pub(crate) use asset_surface::BuiltinAssetSurfaceTemplateBridge;
pub(crate) use floating_window_source::{
    BuiltinFloatingWindowSourceFrames, BuiltinFloatingWindowSourceTemplateBridge,
};
pub(crate) use inspector_surface::BuiltinInspectorSurfaceTemplateBridge;
pub(crate) use pane_surface::BuiltinPaneSurfaceTemplateBridge;
#[cfg(test)]
pub(crate) use projection_support::project_builtin_surface;
pub(crate) use projection_support::{binding_for_control, project_builtin_surface_with_runtime};
pub(crate) use viewport_toolbar::BuiltinViewportToolbarTemplateBridge;
pub(crate) use welcome_surface::BuiltinWelcomeSurfaceTemplateBridge;
pub(crate) use workbench::{
    BuiltinHostOuterShellFrames, BuiltinHostRootShellFrames, BuiltinHostWindowTemplateBridge,
    BuiltinHostWindowTemplateBridgeError, BuiltinWorkbenchWindowLayoutFrames,
    BuiltinWorkbenchWindowTemplateSurfaceBridge, WorkbenchCommandPaletteOpenState,
    WorkbenchSettingsEditorKind, WorkbenchSettingsOpenState, WorkbenchSettingsWindowRevision,
    WORKBENCH_CONTEXT_MENU_CONTROL_ID, WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID,
    WORKBENCH_SETTINGS_WINDOW_CONTROL_ID, WORKBENCH_TOAST_CONTROL_ID,
};

// 启动时只加载宿主、浮窗、工具栏和面板所需的内建文档，供各桥共享同一模板运行时。
pub(crate) fn load_startup_builtin_template_runtime(
) -> Result<EditorUiHostRuntime, EditorUiHostRuntimeError> {
    projection_support::load_builtin_runtime_for_documents(&[
        UNREAL_EDITOR_SHELL_DOCUMENT_ID,
        BUILTIN_FLOATING_WINDOW_SOURCE_DOCUMENT_ID,
        BUILTIN_VIEWPORT_TOOLBAR_DOCUMENT_ID,
        BUILTIN_INSPECTOR_SURFACE_DOCUMENT_ID,
        BUILTIN_PANE_SURFACE_DOCUMENT_ID,
        PANE_HIERARCHY_BODY_DOCUMENT_ID,
        PANE_INSPECTOR_BODY_DOCUMENT_ID,
        PANE_CONSOLE_BODY_DOCUMENT_ID,
        UNREAL_EDITOR_WINDOW_DOCUMENT_ID,
    ])
}
