use zircon_runtime_interface::ui::event_ui::UiNodeId;

pub(crate) const VIEWPORT_SURFACE_ROOT_ID: UiNodeId = UiNodeId::new(1);
pub(crate) const VIEWPORT_SURFACE_NODE_ID: UiNodeId = UiNodeId::new(2);
pub(crate) const BUILTIN_UI_HOST_WINDOW_DOCUMENT_ID: &str = "ui.host_window";
pub(crate) const BUILTIN_HOST_DRAWER_SOURCE_DOCUMENT_ID: &str = "workbench.drawer_source";
pub(crate) const BUILTIN_FLOATING_WINDOW_SOURCE_DOCUMENT_ID: &str = "floating_window.source";
pub(crate) const BUILTIN_VIEWPORT_TOOLBAR_DOCUMENT_ID: &str = "scene.viewport_toolbar";
pub(crate) const BUILTIN_ASSET_SURFACE_DOCUMENT_ID: &str = "asset.surface_controls";
pub(crate) const BUILTIN_WELCOME_SURFACE_DOCUMENT_ID: &str = "startup.welcome_controls";
pub(crate) const BUILTIN_INSPECTOR_SURFACE_DOCUMENT_ID: &str = "inspector.surface_controls";
pub(crate) const BUILTIN_PANE_SURFACE_DOCUMENT_ID: &str = "pane.surface_controls";
pub(crate) const UI_HOST_WINDOW_CONTROL_ID: &str = "UiHostWindowRoot";
pub(crate) const DOCUMENT_TABS_CONTROL_ID: &str = "DocumentTabsRoot";
pub(crate) const PANE_SURFACE_CONTROL_ID: &str = "PaneSurfaceRoot";
