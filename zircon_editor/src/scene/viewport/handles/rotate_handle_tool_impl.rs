use zircon_math::Transform;
use zircon_scene::{HandleOverlayExtract, OverlayAxis, SceneViewportTool, TransformSpace};

use crate::scene::viewport::handles::{
    handle_commit::HandleCommit,
    handle_drag_context::HandleDragContext,
    handle_drag_session::HandleDragSession,
    handle_pick_context::HandlePickContext,
    handle_tool::HandleTool,
    helpers::{
        basis_axis, begin_transform_session, center_anchor, global_rotation_delta,
        local_rotation_delta, maybe_snap, projected_axis_delta, push_axis_ring, selected_basis,
    },
    rotate_handle_tool::RotateHandleTool,
};
use crate::GizmoAxis;

impl HandleTool for RotateHandleTool {
    fn tool(&self) -> SceneViewportTool {
        SceneViewportTool::Rotate
    }

    fn build_overlay(
        &self,
        ctx: &crate::scene::viewport::handles::handle_build_context::HandleBuildContext<'_>,
    ) -> Option<HandleOverlayExtract> {
        let (selected, basis) = selected_basis(ctx)?;
        let mut elements = Vec::new();
        push_axis_ring(
            &mut elements,
            OverlayAxis::X,
            basis.origin.translation,
            basis.x,
            basis.extent * 0.72,
        );
        push_axis_ring(
            &mut elements,
            OverlayAxis::Y,
            basis.origin.translation,
            basis.y,
            basis.extent * 0.72,
        );
        push_axis_ring(
            &mut elements,
            OverlayAxis::Z,
            basis.origin.translation,
            basis.z,
            basis.extent * 0.72,
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
        axis: GizmoAxis,
    ) -> Option<HandleDragSession> {
        Some(HandleDragSession::Rotate(begin_transform_session(
            ctx, axis,
        )?))
    }

    fn update_drag(
        &self,
        session: &mut HandleDragSession,
        ctx: &HandleDragContext<'_>,
    ) -> Option<Transform> {
        let HandleDragSession::Rotate(session) = session else {
            return None;
        };
        let scalar = projected_axis_delta(
            session.start_cursor,
            ctx.current_cursor,
            session.initial_transform.translation,
            basis_axis(&session.basis, session.axis),
            ctx.camera,
            ctx.viewport,
        )?;
        let angle = maybe_snap(
            scalar * 0.01,
            session.snap_enabled,
            session.rotate_step_radians.max(0.0001),
        );
        let rotation = match session.space {
            TransformSpace::Local => {
                session.initial_transform.rotation * local_rotation_delta(session.axis, angle)
            }
            TransformSpace::Global => {
                global_rotation_delta(session.axis, angle) * session.initial_transform.rotation
            }
        };
        Some(Transform {
            rotation,
            ..session.initial_transform
        })
    }

    fn end_drag(&self, session: HandleDragSession) -> Option<HandleCommit> {
        let HandleDragSession::Rotate(session) = session else {
            return None;
        };
        Some(HandleCommit {
            node_id: session.node_id,
            tool: self.tool(),
            initial_transform: session.initial_transform,
        })
    }
}
