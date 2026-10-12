use zircon_runtime_interface::ui::event_ui::UiNodeId;

pub(super) const ROOT_NODE_ID: UiNodeId = UiNodeId::new(1);
pub(super) const SURFACE_NODE_ID_BASE: u64 = 10;
pub(super) const CONTROL_NODE_ID_BASE: u64 = 100;
pub(super) const VIEWPORT_TOOLBAR_ROUTE_ID_BASE: u64 = 59_000;
pub(super) const VIEWPORT_TOOLBAR_SURFACE_STRIDE: u64 = 1_000;
pub(super) const SURFACE_VERTICAL_STRIDE: f32 = 64.0;

#[cfg(test)]
pub(super) const SURFACE_WIDTH: f32 = 1024.0;
#[cfg(test)]
pub(super) const SURFACE_HEIGHT: f32 = 32.0;
