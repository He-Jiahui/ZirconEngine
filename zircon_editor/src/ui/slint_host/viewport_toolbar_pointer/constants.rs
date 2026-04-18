use zircon_ui::UiNodeId;

pub(super) const ROOT_NODE_ID: UiNodeId = UiNodeId::new(1);
pub(super) const SURFACE_NODE_ID_BASE: u64 = 10;
pub(super) const CONTROL_NODE_ID_BASE: u64 = 100;
pub(super) const SURFACE_VERTICAL_STRIDE: f32 = 64.0;

#[cfg(test)]
pub(super) const SURFACE_WIDTH: f32 = 1024.0;
#[cfg(test)]
pub(super) const SURFACE_HEIGHT: f32 = 32.0;
