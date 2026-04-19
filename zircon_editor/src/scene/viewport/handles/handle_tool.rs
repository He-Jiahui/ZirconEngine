use crate::GizmoAxis;
use zircon_framework::render::{HandleOverlayExtract, SceneViewportTool};
use zircon_math::Transform;

use super::{
    handle_build_context::HandleBuildContext, handle_drag_context::HandleDragContext,
    handle_drag_session::HandleDragSession, handle_pick_context::HandlePickContext,
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
    fn end_drag(&self, session: HandleDragSession);
}
