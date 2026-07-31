use crate::scene::viewport::GizmoAxis;
use crate::scene::viewport::{
    HandleOverlayExtract, SceneViewportSettings, SceneViewportSnapSteps, TransformHandleKind,
    ViewportCameraSnapshot,
};
use zircon_runtime::scene::Scene;
use zircon_runtime_interface::math::{Transform, UVec2, Vec2};

use super::{
    handle_drag_context::HandleDragContext, handle_drag_session::HandleDragSession,
    handle_pick_context::HandlePickContext, handle_tool::HandleTool,
    handle_tool_registry::HandleToolRegistry,
};

impl HandleToolRegistry {
    pub(crate) fn build_overlays(
        &self,
        scene: &Scene,
        selected: Option<u64>,
        settings: &SceneViewportSettings,
        handle_kind: Option<TransformHandleKind>,
        camera: &ViewportCameraSnapshot,
    ) -> Vec<HandleOverlayExtract> {
        let Some(tool) = handle_kind.and_then(|kind| self.tool(kind)) else {
            return Vec::new();
        };
        tool.build_overlay(&super::handle_build_context::HandleBuildContext {
            scene,
            selected,
            settings,
            camera,
        })
        .into_iter()
        .collect()
    }

    pub(crate) fn begin_drag(
        &self,
        scene: &Scene,
        selected: Option<u64>,
        settings: &SceneViewportSettings,
        handle_kind: Option<TransformHandleKind>,
        snap_steps: SceneViewportSnapSteps,
        camera: &ViewportCameraSnapshot,
        cursor: Vec2,
        axis: GizmoAxis,
    ) -> Option<HandleDragSession> {
        let tool = self.tool(handle_kind?)?;
        tool.begin_drag(
            &HandlePickContext {
                scene,
                selected,
                settings,
                snap_steps,
                camera,
                cursor,
            },
            axis,
        )
    }

    pub(crate) fn update_drag(
        &self,
        session: &mut HandleDragSession,
        camera: &ViewportCameraSnapshot,
        viewport: UVec2,
        current_cursor: Vec2,
    ) -> Option<Transform> {
        self.tool(session.kind())?.update_drag(
            session,
            &HandleDragContext {
                camera,
                viewport,
                current_cursor,
            },
        )
    }

    pub(crate) fn end_drag(&self, session: HandleDragSession) {
        if let Some(tool) = self.tool(session.kind()) {
            tool.end_drag(session);
        }
    }

    fn tool(&self, kind: TransformHandleKind) -> Option<&dyn HandleTool> {
        match kind {
            TransformHandleKind::Move => Some(&self.move_tool),
            TransformHandleKind::Rotate => Some(&self.rotate_tool),
            TransformHandleKind::Scale => Some(&self.scale_tool),
        }
    }
}
