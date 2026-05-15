mod asset_surface;
mod floating_window_source;
mod inspector_surface;
mod pane_surface;
mod projection_support;
mod viewport_toolbar;
mod welcome_surface;
mod workbench;
mod workbench_drawer_source;

use crate::ui::template_runtime::builtin::{
    PANE_CONSOLE_BODY_DOCUMENT_ID, PANE_HIERARCHY_BODY_DOCUMENT_ID, PANE_INSPECTOR_BODY_DOCUMENT_ID,
};
use crate::ui::template_runtime::{EditorUiHostRuntime, EditorUiHostRuntimeError};

use super::constants::{
    BUILTIN_FLOATING_WINDOW_SOURCE_DOCUMENT_ID, BUILTIN_HOST_DRAWER_SOURCE_DOCUMENT_ID,
    BUILTIN_INSPECTOR_SURFACE_DOCUMENT_ID, BUILTIN_PANE_SURFACE_DOCUMENT_ID,
    BUILTIN_UI_HOST_WINDOW_DOCUMENT_ID, BUILTIN_VIEWPORT_TOOLBAR_DOCUMENT_ID,
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
pub(crate) use workbench::{BuiltinHostRootShellFrames, BuiltinHostWindowTemplateBridge};
#[cfg(test)]
pub(crate) use workbench_drawer_source::BuiltinHostDrawerSourceTemplateBridge;

pub(crate) fn load_startup_builtin_template_runtime(
) -> Result<EditorUiHostRuntime, EditorUiHostRuntimeError> {
    projection_support::load_builtin_runtime_for_documents(&[
        BUILTIN_UI_HOST_WINDOW_DOCUMENT_ID,
        BUILTIN_HOST_DRAWER_SOURCE_DOCUMENT_ID,
        BUILTIN_FLOATING_WINDOW_SOURCE_DOCUMENT_ID,
        BUILTIN_VIEWPORT_TOOLBAR_DOCUMENT_ID,
        BUILTIN_INSPECTOR_SURFACE_DOCUMENT_ID,
        BUILTIN_PANE_SURFACE_DOCUMENT_ID,
        PANE_HIERARCHY_BODY_DOCUMENT_ID,
        PANE_INSPECTOR_BODY_DOCUMENT_ID,
        PANE_CONSOLE_BODY_DOCUMENT_ID,
    ])
}
