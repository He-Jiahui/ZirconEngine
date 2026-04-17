use crate::scene::scene_renderer::overlay::{PreparedIconDraw, PreparedSceneGizmoPass};
use crate::scene::scene_renderer::primitives::{
    build_icon_buffer, build_icon_quad_vertices, build_line_buffer, build_scene_gizmo_line_vertices,
};
use crate::types::{EditorOrRuntimeFrame, GraphicsError};

use super::scene_gizmo_pass::SceneGizmoPass;

impl SceneGizmoPass {
    pub(crate) fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
        frame: &EditorOrRuntimeFrame,
    ) -> Result<PreparedSceneGizmoPass, GraphicsError> {
        let camera = &frame.scene.scene.camera;
        let camera_right = camera.transform.right();
        let camera_up = camera.transform.up();
        let mut icon_draws = Vec::new();

        for gizmo in &frame.scene.overlays.scene_gizmos {
            for icon in &gizmo.icons {
                let Some(bind_group) = self.icon_atlas.ensure(
                    icon.id,
                    device,
                    queue,
                    texture_layout,
                    &self.icon_sampler,
                )?
                else {
                    continue;
                };
                let vertices = build_icon_quad_vertices(icon, camera_right, camera_up);
                if let Some((vertex_buffer, vertex_count)) =
                    build_icon_buffer(device, "zircon-scene-gizmo-icon-buffer", &vertices)
                {
                    icon_draws.push(PreparedIconDraw {
                        bind_group,
                        vertex_buffer,
                        vertex_count,
                    });
                }
            }
        }

        let line_vertices = build_scene_gizmo_line_vertices(frame, |id| self.icon_atlas.has(id));
        Ok(PreparedSceneGizmoPass {
            line_buffer: build_line_buffer(device, "zircon-scene-gizmo-buffer", &line_vertices),
            icon_draws,
        })
    }
}
