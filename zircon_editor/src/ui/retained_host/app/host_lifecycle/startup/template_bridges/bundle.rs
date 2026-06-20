use std::sync::Arc;

use super::super::super::super::*;

pub(in crate::ui::retained_host::app::host_lifecycle::startup) struct StartupTemplateBridges {
    pub(in crate::ui::retained_host::app::host_lifecycle::startup) builtin_template_runtime:
        Arc<EditorUiHostRuntime>,
    pub(in crate::ui::retained_host::app::host_lifecycle::startup) template_bridge:
        callback_dispatch::BuiltinHostWindowTemplateBridge,
    pub(in crate::ui::retained_host::app::host_lifecycle::startup) workbench_window_bridge:
        callback_dispatch::BuiltinWorkbenchWindowTemplateSurfaceBridge,
    pub(in crate::ui::retained_host::app::host_lifecycle::startup) floating_window_source_bridge:
        callback_dispatch::BuiltinFloatingWindowSourceTemplateBridge,
    pub(in crate::ui::retained_host::app::host_lifecycle::startup) viewport_toolbar_bridge:
        callback_dispatch::BuiltinViewportToolbarTemplateBridge,
    pub(in crate::ui::retained_host::app::host_lifecycle::startup) inspector_surface_bridge:
        callback_dispatch::BuiltinInspectorSurfaceTemplateBridge,
    pub(in crate::ui::retained_host::app::host_lifecycle::startup) pane_surface_bridge:
        callback_dispatch::BuiltinPaneSurfaceTemplateBridge,
    pub(in crate::ui::retained_host::app::host_lifecycle::startup) component_showcase_runtime:
        EditorUiHostRuntime,
}
