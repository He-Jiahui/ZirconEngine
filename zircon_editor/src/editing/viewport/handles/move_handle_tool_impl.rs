use zircon_math::Transform;
use zircon_scene::{HandleOverlayExtract, OverlayAxis, SceneViewportTool};

use crate::editing::viewport::handles::{
    handle_commit::HandleCommit,
    handle_drag_context::HandleDragContext,
    handle_drag_session::HandleDragSession,
    handle_pick_context::HandlePickContext,
    handle_tool::HandleTool,
    helpers::{
        basis_axis, begin_transform_session, center_anchor, maybe_snap, projected_axis_delta,
        push_axis_line, selected_basis,
    },
    move_handle_tool::MoveHandleTool,
};
use crate::editing::viewport::projection::world_units_per_pixel;

impl HandleTool for MoveHandleTool {
    fn tool(&self) -> SceneViewportTool {
        SceneViewportTool::Move
    }

    fn build_overlay(
        &self,
        ctx: &crate::editing::viewport::handles::handle_build_context::HandleBuildContext<'_>,
    ) -> Option<HandleOverlayExtract> {
        let (selected, basis) = selected_basis(ctx)?;
        let mut elements = Vec::new();
        push_axis_line(
            &mut elements,
            OverlayAxis::X,
            basis.origin.translation,
            basis.x,
            basis.extent,
        );
        push_axis_line(
            &mut elements,
            OverlayAxis::Y,
            basis.origin.translation,
            basis.y,
            basis.extent,
        );
        push_axis_line(
            &mut elements,
            OverlayAxis::Z,
            basis.origin.translation,
            basis.z,
            basis.extent,
        );
        elements.push(center_anchor(&basis));
        Some(HandleOverlayExtract {
            owner: selected,
            tool: self.tool(),
            space: ctx.settings.transform_space,
            origin: basis.origin,
            elements,
        })
    }

    fn begin_drag(
        &self,
        ctx: &HandlePickContext<'_>,
        axis: zircon_graphics::GizmoAxis,
    ) -> Option<HandleDragSession> {
        Some(HandleDragSession::Move(begin_transform_session(ctx, axis)?))
    }

    fn update_drag(
        &self,
        session: &mut HandleDragSession,
        ctx: &HandleDragContext<'_>,
    ) -> Option<Transform> {
        let HandleDragSession::Move(session) = session else {
            return None;
        };
        let axis_vector = basis_axis(&session.basis, session.axis);
        let scalar = projected_axis_delta(
            session.start_cursor,
            ctx.current_cursor,
            session.initial_transform.translation,
            axis_vector,
            ctx.camera,
            ctx.viewport,
        )?;
        let amount = maybe_snap(
            scalar
                * world_units_per_pixel(
                    ctx.camera,
                    session.initial_transform.translation,
                    ctx.viewport,
                ),
            session.snap_enabled,
            session.translate_step.max(0.0001),
        );
        Some(Transform {
            translation: session.initial_transform.translation + axis_vector * amount,
            ..session.initial_transform
        })
    }

    fn end_drag(&self, session: HandleDragSession) -> Option<HandleCommit> {
        let HandleDragSession::Move(session) = session else {
            return None;
        };
        Some(HandleCommit {
            node_id: session.node_id,
            tool: self.tool(),
            initial_transform: session.initial_transform,
        })
    }
}
