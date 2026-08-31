use crate::scene::viewport::GizmoAxis;
use crate::scene::viewport::GridMode;

use crate::scene::viewport::handles::{
    handle_basis::HandleBasis, handle_build_context::HandleBuildContext,
    handle_pick_context::HandlePickContext, helpers::build_handle_basis,
    transform_handle_drag_session::TransformHandleDragSession,
};

pub(in crate::scene::viewport::handles) fn selected_basis(
    ctx: &HandleBuildContext<'_>,
) -> Option<(u64, HandleBasis)> {
    let selected = ctx.selected?;
    Some((
        selected.entity,
        build_handle_basis(selected.transform, ctx.settings.transform_space, ctx.camera),
    ))
}

pub(in crate::scene::viewport::handles) fn begin_transform_session(
    ctx: &HandlePickContext<'_>,
    axis: GizmoAxis,
) -> Option<TransformHandleDragSession> {
    let selected = ctx.selected?;
    Some(TransformHandleDragSession {
        node_id: selected.entity,
        axis,
        start_cursor: ctx.cursor,
        initial_transform: selected.transform,
        basis: build_handle_basis(selected.transform, ctx.settings.transform_space, ctx.camera),
        space: ctx.settings.transform_space,
        snap_enabled: ctx.settings.grid_mode == GridMode::VisibleAndSnap,
        translate_step: ctx.snap_steps.translate_step,
        rotate_step_radians: ctx.snap_steps.rotate_step_deg.to_radians(),
        scale_step: ctx.snap_steps.scale_step,
    })
}
