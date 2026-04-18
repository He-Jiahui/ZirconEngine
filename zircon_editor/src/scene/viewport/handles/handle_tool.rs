use crate::GizmoAxis;
use zircon_math::Transform;
use zircon_scene::{HandleOverlayExtract, SceneViewportTool};

use super::{
    handle_build_context::HandleBuildContext, handle_commit::HandleCommit,
    handle_drag_context::HandleDragContext, handle_drag_session::HandleDragSession,
    handle_pick_context::HandlePickContext,
};

pub(crate) trait HandleTool {
    fn tool(&self) -> SceneViewportTool;
    fn build_overlay(&self, ctx: &HandleBuildContext<'_>) -> Option<HandleOverlayExtract>;
    fn begin_drag(&self, ctx: &HandlePickContext<'_>, axis: GizmoAxis)
        -> Option<HandleDragSession>;
    fn update_drag(
        &self,
        session: &mut HandleDragSession,
        ctx: &HandleDragContext<'_>,
    ) -> Option<Transform>;
    fn end_drag(&self, session: HandleDragSession) -> Option<HandleCommit>;
}
