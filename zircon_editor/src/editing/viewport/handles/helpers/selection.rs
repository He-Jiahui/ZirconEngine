use zircon_graphics::GizmoAxis;
use zircon_scene::GridMode;

use crate::editing::viewport::handles::{
    handle_basis::HandleBasis, handle_build_context::HandleBuildContext,
    handle_pick_context::HandlePickContext, helpers::build_handle_basis,
    transform_handle_drag_session::TransformHandleDragSession,
};

pub(in crate::editing::viewport::handles) fn selected_basis(
    ctx: &HandleBuildContext<'_>,
) -> Option<(u64, HandleBasis)> {
    let selected = ctx.scene.selected_node()?;
    let node = ctx.scene.find_node(selected)?;
    Some((
        selected,
        build_handle_basis(node.transform, ctx.settings.transform_space, ctx.camera),
    ))
}

pub(in crate::editing::viewport::handles) fn begin_transform_session(
    ctx: &HandlePickContext<'_>,
    axis: GizmoAxis,
) -> Option<TransformHandleDragSession> {
    let selected = ctx.scene.selected_node()?;
    let node = ctx.scene.find_node(selected)?;
    Some(TransformHandleDragSession {
        node_id: selected,
        axis,
        start_cursor: ctx.cursor,
        initial_transform: node.transform,
        basis: build_handle_basis(node.transform, ctx.settings.transform_space, ctx.camera),
        space: ctx.settings.transform_space,
        snap_enabled: ctx.settings.grid_mode == GridMode::VisibleAndSnap,
        translate_step: ctx.settings.translate_step,
        rotate_step_radians: ctx.settings.rotate_step_deg.to_radians(),
        scale_step: ctx.settings.scale_step,
    })
}
