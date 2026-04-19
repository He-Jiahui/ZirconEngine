use zircon_runtime::core::math::{UVec2, Vec3};

use super::drag_state::DragState;

#[derive(Clone, Debug)]
pub(in crate::entry::runtime_entry_app) struct RuntimeCameraController {
    pub(super) viewport_size: UVec2,
    pub(super) orbit_target: Vec3,
    pub(super) drag: Option<DragState>,
}
