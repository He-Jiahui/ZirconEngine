use crate::GizmoAxis;
use zircon_math::{Transform, Vec2};
use zircon_scene::TransformSpace;

use super::handle_basis::HandleBasis;

#[derive(Clone, Debug)]
pub(in crate::editing::viewport) struct TransformHandleDragSession {
    pub(in crate::editing::viewport::handles) node_id: u64,
    pub(in crate::editing::viewport::handles) axis: GizmoAxis,
    pub(in crate::editing::viewport::handles) start_cursor: Vec2,
    pub(in crate::editing::viewport::handles) initial_transform: Transform,
    pub(in crate::editing::viewport::handles) basis: HandleBasis,
    pub(in crate::editing::viewport::handles) space: TransformSpace,
    pub(in crate::editing::viewport::handles) snap_enabled: bool,
    pub(in crate::editing::viewport::handles) translate_step: f32,
    pub(in crate::editing::viewport::handles) rotate_step_radians: f32,
    pub(in crate::editing::viewport::handles) scale_step: f32,
}
