use crate::graphics::scene::scene_renderer::overlay::{PreparedIconDraw, PreparedSceneGizmoPass};
use crate::graphics::scene::scene_renderer::primitives::{
    build_icon_buffer, build_icon_quad_vertices, build_line_buffer, build_scene_gizmo_line_vertices,
};
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};
use zr_rhi_wgpu::WgpuTextureUploadBatch;

use super::scene_gizmo_pass::SceneGizmoPass;

impl SceneGizmoPass {
    pub(crate) fn prepare(
        &mut self,
        device: &wgpu::Device,
        texture_layout: &wgpu::BindGroupLayout,
        frame: &ViewportRenderFrame,
        frame_texture_uploads: &mut WgpuTextureUploadBatch,
    ) -> Result<PreparedSceneGizmoPass, GraphicsError> {
        let camera = frame.effective_camera();
        let camera_right = camera.transform.right();
        let camera_up = camera.transform.up();
        let mut icon_draws = Vec::new();

        for gizmo in &frame.overlays().scene_gizmos {
            for icon in &gizmo.icons {
                let Some(bind_group) =
                    self.icon_atlas
                        .ensure(icon.id, device, texture_layout, &self.icon_sampler)?
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
        self.icon_atlas
            .append_pending_uploads(frame_texture_uploads);

        let line_vertices = build_scene_gizmo_line_vertices(frame, |id| self.icon_atlas.has(id));
        Ok(PreparedSceneGizmoPass {
            line_buffer: build_line_buffer(device, "zircon-scene-gizmo-buffer", &line_vertices),
            icon_draws,
        })
    }
}

#[cfg(test)]
mod tests {
    const SOURCE: &str = include_str!("prepare.rs");

    #[test]
    fn scene_gizmo_appends_pending_icon_uploads_once_after_icon_discovery() {
        let production = SOURCE
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("scene gizmo prepare source should retain a test-module boundary");
        let discovery = production
            .find("for gizmo in &frame.overlays().scene_gizmos")
            .expect("scene gizmo icon discovery");
        let append = production
            .find(".append_pending_uploads(frame_texture_uploads)")
            .expect("pending icon upload append");

        assert!(discovery < append);
        assert_eq!(
            production
                .matches(".append_pending_uploads(frame_texture_uploads)")
                .count(),
            1
        );
        assert!(!production.contains("wgpu::Queue"));
    }
}
