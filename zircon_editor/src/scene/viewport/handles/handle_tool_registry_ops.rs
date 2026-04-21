use crate::scene::viewport::GizmoAxis;
use crate::scene::viewport::{
    HandleOverlayExtract, SceneViewportSettings, SceneViewportTool, ViewportCameraSnapshot,
};
use zircon_runtime::core::math::{Transform, UVec2, Vec2};
use zircon_runtime::scene::Scene;

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
        camera: &ViewportCameraSnapshot,
    ) -> Vec<HandleOverlayExtract> {
        let Some(tool) = self.tool(settings.tool) else {
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
        camera: &ViewportCameraSnapshot,
        cursor: Vec2,
        axis: GizmoAxis,
    ) -> Option<HandleDragSession> {
        let tool = self.tool(settings.tool)?;
        tool.begin_drag(
            &HandlePickContext {
                scene,
                selected,
                settings,
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
        self.tool(session.tool())?.update_drag(
            session,
            &HandleDragContext {
                camera,
                viewport,
                current_cursor,
            },
        )
    }

    pub(crate) fn end_drag(&self, session: HandleDragSession) {
        if let Some(tool) = self.tool(session.tool()) {
            tool.end_drag(session);
        }
    }

    fn tool(&self, tool: SceneViewportTool) -> Option<&dyn HandleTool> {
        match tool {
            SceneViewportTool::Drag => None,
            SceneViewportTool::Move => Some(&self.move_tool),
            SceneViewportTool::Rotate => Some(&self.rotate_tool),
            SceneViewportTool::Scale => Some(&self.scale_tool),
        }
    }
}
