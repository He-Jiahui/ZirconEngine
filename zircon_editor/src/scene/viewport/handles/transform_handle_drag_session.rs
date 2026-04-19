use crate::GizmoAxis;
use crate::scene::viewport::TransformSpace;
use zircon_runtime::core::math::{Transform, Vec2};

use super::handle_basis::HandleBasis;

#[derive(Clone, Debug)]
pub(in crate::scene::viewport) struct TransformHandleDragSession {
    pub(in crate::scene::viewport::handles) node_id: u64,
    pub(in crate::scene::viewport::handles) axis: GizmoAxis,
    pub(in crate::scene::viewport::handles) start_cursor: Vec2,
    pub(in crate::scene::viewport::handles) initial_transform: Transform,
    pub(in crate::scene::viewport::handles) basis: HandleBasis,
    pub(in crate::scene::viewport::handles) space: TransformSpace,
    pub(in crate::scene::viewport::handles) snap_enabled: bool,
    pub(in crate::scene::viewport::handles) translate_step: f32,
    pub(in crate::scene::viewport::handles) rotate_step_radians: f32,
    pub(in crate::scene::viewport::handles) scale_step: f32,
}
