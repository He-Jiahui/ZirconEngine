mod asset_surface;
mod floating_window_source;
mod inspector_surface;
mod pane_surface;
mod projection_support;
mod viewport_toolbar;
mod welcome_surface;
mod workbench;
mod workbench_drawer_source;

pub(crate) use asset_surface::BuiltinAssetSurfaceTemplateBridge;
pub(crate) use floating_window_source::{
    BuiltinFloatingWindowSourceFrames, BuiltinFloatingWindowSourceTemplateBridge,
};
pub(crate) use inspector_surface::BuiltinInspectorSurfaceTemplateBridge;
pub(crate) use pane_surface::BuiltinPaneSurfaceTemplateBridge;
pub(crate) use projection_support::{binding_for_control, project_builtin_surface};
pub(crate) use viewport_toolbar::BuiltinViewportToolbarTemplateBridge;
pub(crate) use welcome_surface::BuiltinWelcomeSurfaceTemplateBridge;
pub(crate) use workbench::{BuiltinHostRootShellFrames, BuiltinHostWindowTemplateBridge};
#[cfg(test)]
pub(crate) use workbench_drawer_source::BuiltinHostDrawerSourceTemplateBridge;
