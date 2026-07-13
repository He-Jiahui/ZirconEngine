use zircon_runtime_interface::ui::event_ui::UiNodeId;

pub(crate) const VIEWPORT_SURFACE_ROOT_ID: UiNodeId = UiNodeId::new(1);
pub(crate) const VIEWPORT_SURFACE_NODE_ID: UiNodeId = UiNodeId::new(2);
pub(crate) const BUILTIN_UI_HOST_WINDOW_DOCUMENT_ID: &str =
    "res://ui/editor/host/workbench_shell.zui";
pub(crate) const BUILTIN_FLOATING_WINDOW_SOURCE_DOCUMENT_ID: &str =
    "res://ui/editor/host/floating_window_source.zui";
pub(crate) const BUILTIN_VIEWPORT_TOOLBAR_DOCUMENT_ID: &str =
    "res://ui/editor/host/scene_viewport_toolbar.zui";
pub(crate) const BUILTIN_ASSET_SURFACE_DOCUMENT_ID: &str =
    "res://ui/editor/host/asset_surface_controls.zui";
pub(crate) const BUILTIN_WELCOME_SURFACE_DOCUMENT_ID: &str =
    "res://ui/editor/host/startup_welcome_controls.zui";
pub(crate) const BUILTIN_INSPECTOR_SURFACE_DOCUMENT_ID: &str =
    "res://ui/editor/host/inspector_surface_controls.zui";
pub(crate) const BUILTIN_PANE_SURFACE_DOCUMENT_ID: &str =
    "res://ui/editor/host/pane_surface_controls.zui";
pub(crate) const UI_HOST_WINDOW_CONTROL_ID: &str = "UiHostWindowRoot";
pub(crate) const DOCUMENT_TABS_CONTROL_ID: &str = "DocumentTabsRoot";
pub(crate) const PANE_SURFACE_CONTROL_ID: &str = "PaneSurfaceRoot";
