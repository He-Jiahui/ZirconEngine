use zircon_math::Transform;
use zircon_scene::{HandleOverlayExtract, OverlayAxis, SceneViewportTool};

use crate::scene::viewport::handles::{
    handle_commit::HandleCommit,
    handle_drag_context::HandleDragContext,
    handle_drag_session::HandleDragSession,
    handle_pick_context::HandlePickContext,
    handle_tool::HandleTool,
    helpers::{
        basis_axis, begin_transform_session, center_anchor, maybe_snap, projected_axis_delta,
        push_axis_scale, selected_basis,
    },
    scale_handle_tool::ScaleHandleTool,
};
use crate::GizmoAxis;

impl HandleTool for ScaleHandleTool {
    fn tool(&self) -> SceneViewportTool {
        SceneViewportTool::Scale
    }

    fn build_overlay(
        &self,
        ctx: &crate::scene::viewport::handles::handle_build_context::HandleBuildContext<'_>,
    ) -> Option<HandleOverlayExtract> {
        let (selected, basis) = selected_basis(ctx)?;
        let mut elements = Vec::new();
        push_axis_scale(
            &mut elements,
            OverlayAxis::X,
            basis.origin.translation,
            basis.x,
            basis.extent,
        );
        push_axis_scale(
            &mut elements,
            OverlayAxis::Y,
            basis.origin.translation,
            basis.y,
            basis.extent,
        );
        push_axis_scale(
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
        axis: GizmoAxis,
    ) -> Option<HandleDragSession> {
        Some(HandleDragSession::Scale(begin_transform_session(
            ctx, axis,
        )?))
    }

    fn update_drag(
        &self,
        session: &mut HandleDragSession,
        ctx: &HandleDragContext<'_>,
    ) -> Option<Transform> {
        let HandleDragSession::Scale(session) = session else {
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
        let delta = maybe_snap(
            scalar * 0.01,
            session.snap_enabled,
            session.scale_step.max(0.0001),
        );
        let mut scale = session.initial_transform.scale;
        match session.axis {
            GizmoAxis::X => scale.x = (session.initial_transform.scale.x + delta).max(0.05),
            GizmoAxis::Y => scale.y = (session.initial_transform.scale.y + delta).max(0.05),
            GizmoAxis::Z => scale.z = (session.initial_transform.scale.z + delta).max(0.05),
        }
        Some(Transform {
            scale,
            ..session.initial_transform
        })
    }

    fn end_drag(&self, session: HandleDragSession) -> Option<HandleCommit> {
        let HandleDragSession::Scale(session) = session else {
            return None;
        };
        Some(HandleCommit {
            node_id: session.node_id,
            tool: self.tool(),
            initial_transform: session.initial_transform,
        })
    }
}
